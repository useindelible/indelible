use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use ind_application::AppError;
use ind_application::repos::Cursor;
use ind_domain::DomainError;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

fn cursor_error(msg: impl Into<String>) -> AppError {
    AppError::Domain(DomainError::Validation {
        field: "cursor".into(),
        message: msg.into(),
    })
}

pub(crate) fn encode_cursor_ts(ts: DateTime<Utc>, id: Uuid) -> Cursor {
    let payload = format!("{}|{}", ts.to_rfc3339(), id);
    Cursor(URL_SAFE_NO_PAD.encode(payload))
}

pub(crate) fn decode_cursor_ts(cursor: &Cursor) -> Result<(DateTime<Utc>, Uuid), AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&cursor.0)
        .map_err(|_| cursor_error("invalid base64 encoding"))?;
    let payload =
        String::from_utf8(bytes).map_err(|_| cursor_error("cursor contains invalid UTF-8"))?;

    let (ts_str, id_str) = payload
        .split_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;

    let ts: DateTime<Utc> = ts_str
        .parse()
        .map_err(|_| cursor_error("invalid timestamp in cursor"))?;
    let id: Uuid = id_str
        .parse()
        .map_err(|_| cursor_error("invalid UUID in cursor"))?;

    Ok((ts, id))
}

pub(crate) fn encode_cursor_collection(sort_order: i32, name: &str, id: Uuid) -> Cursor {
    let payload = format!("{sort_order}|{name}|{id}");
    Cursor(URL_SAFE_NO_PAD.encode(payload))
}

pub(crate) fn decode_cursor_collection(cursor: &Cursor) -> Result<(i32, String, Uuid), AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&cursor.0)
        .map_err(|_| cursor_error("invalid base64 encoding"))?;
    let payload =
        String::from_utf8(bytes).map_err(|_| cursor_error("cursor contains invalid UTF-8"))?;

    // Split from the right: UUID is always last (fixed format, no |)
    let (prefix, id_str) = payload
        .rsplit_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;
    let (sort_str, name) = prefix
        .split_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;

    let sort_order: i32 = sort_str
        .parse()
        .map_err(|_| cursor_error("invalid sort_order in cursor"))?;
    let id: Uuid = id_str
        .parse()
        .map_err(|_| cursor_error("invalid UUID in cursor"))?;

    Ok((sort_order, name.to_owned(), id))
}

pub(crate) fn encode_cursor_name(name: &str, id: Uuid) -> Cursor {
    let payload = format!("{name}|{id}");
    Cursor(URL_SAFE_NO_PAD.encode(payload))
}

pub(crate) fn decode_cursor_name(cursor: &Cursor) -> Result<(String, Uuid), AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&cursor.0)
        .map_err(|_| cursor_error("invalid base64 encoding"))?;
    let payload =
        String::from_utf8(bytes).map_err(|_| cursor_error("cursor contains invalid UTF-8"))?;

    let (name, id_str) = payload
        .rsplit_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;
    let id: Uuid = id_str
        .parse()
        .map_err(|_| cursor_error("invalid UUID in cursor"))?;

    Ok((name.to_owned(), id))
}

pub(crate) fn encode_cursor_entity(
    total_mentions: i64,
    item_count: i64,
    name: &str,
    id: Uuid,
) -> Cursor {
    let payload = format!("{total_mentions}|{item_count}|{name}|{id}");
    Cursor(URL_SAFE_NO_PAD.encode(payload))
}

pub(crate) fn decode_cursor_entity(cursor: &Cursor) -> Result<(i64, i64, String, Uuid), AppError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&cursor.0)
        .map_err(|_| cursor_error("invalid base64 encoding"))?;
    let payload =
        String::from_utf8(bytes).map_err(|_| cursor_error("cursor contains invalid UTF-8"))?;

    // Format: total_mentions|item_count|name|uuid
    // UUID is always the rightmost segment. The name field may contain '|',
    // so we split from the right for the UUID and from the left for the two
    // integer prefixes, leaving everything in between as the name.
    let (prefix, id_str) = payload
        .rsplit_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;
    let (total_mentions_str, rest) = prefix
        .split_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;
    let (item_count_str, name) = rest
        .split_once('|')
        .ok_or_else(|| cursor_error("missing separator in cursor"))?;

    let total_mentions = total_mentions_str
        .parse()
        .map_err(|_| cursor_error("invalid mention count in cursor"))?;
    let item_count = item_count_str
        .parse()
        .map_err(|_| cursor_error("invalid item count in cursor"))?;
    let id = id_str
        .parse()
        .map_err(|_| cursor_error("invalid UUID in cursor"))?;

    Ok((total_mentions, item_count, name.to_owned(), id))
}

pub(crate) fn clamp_limit(requested: u32) -> i64 {
    if requested == 0 {
        DEFAULT_LIMIT
    } else {
        (requested as i64).min(MAX_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(value: &str) -> Cursor {
        Cursor(URL_SAFE_NO_PAD.encode(value))
    }

    #[test]
    fn every_cursor_shape_round_trips_delimiter_bearing_names() {
        let id = Uuid::now_v7();
        let timestamp = Utc::now();

        assert_eq!(
            decode_cursor_ts(&encode_cursor_ts(timestamp, id)).unwrap(),
            (timestamp, id)
        );
        assert_eq!(
            decode_cursor_collection(&encode_cursor_collection(7, "Research | Rust", id)).unwrap(),
            (7, "Research | Rust".into(), id)
        );
        assert_eq!(
            decode_cursor_name(&encode_cursor_name("systems | rust", id)).unwrap(),
            ("systems | rust".into(), id)
        );
        assert_eq!(
            decode_cursor_entity(&encode_cursor_entity(42, 3, "Rust | Foundation", id)).unwrap(),
            (42, 3, "Rust | Foundation".into(), id)
        );
    }

    #[test]
    fn malformed_cursors_are_rejected_at_each_typed_boundary() {
        assert!(decode_cursor_ts(&Cursor("not base64!".into())).is_err());
        assert!(decode_cursor_ts(&encoded("missing-separator")).is_err());
        assert!(decode_cursor_ts(&encoded("not-a-date|not-a-uuid")).is_err());
        assert!(decode_cursor_ts(&encoded("2026-01-01T00:00:00Z|not-a-uuid")).is_err());

        assert!(decode_cursor_collection(&encoded("missing")).is_err());
        assert!(decode_cursor_collection(&encoded("1|missing-uuid")).is_err());
        assert!(decode_cursor_collection(&encoded("bad|name|not-a-uuid")).is_err());
        assert!(decode_cursor_collection(&encoded("1|name|not-a-uuid")).is_err());

        assert!(decode_cursor_name(&encoded("missing")).is_err());
        assert!(decode_cursor_name(&encoded("name|not-a-uuid")).is_err());

        assert!(decode_cursor_entity(&encoded("missing")).is_err());
        assert!(decode_cursor_entity(&encoded("1|missing")).is_err());
        assert!(decode_cursor_entity(&encoded("1|2|missing-uuid")).is_err());
        assert!(decode_cursor_entity(&encoded("bad|2|name|not-a-uuid")).is_err());
        assert!(decode_cursor_entity(&encoded("1|bad|name|not-a-uuid")).is_err());
        assert!(decode_cursor_entity(&encoded("1|2|name|not-a-uuid")).is_err());
    }

    #[test]
    fn limits_have_a_product_default_and_hard_ceiling() {
        assert_eq!(clamp_limit(0), 50);
        assert_eq!(clamp_limit(1), 1);
        assert_eq!(clamp_limit(200), 200);
        assert_eq!(clamp_limit(u32::MAX), 200);
    }
}
