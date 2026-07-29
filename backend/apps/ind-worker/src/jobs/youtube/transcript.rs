use serde::Deserialize;

#[derive(Debug, Clone)]
pub(super) struct TranscriptSegment {
    pub(super) start_ms: i64,
    pub(super) end_ms: Option<i64>,
    pub(super) text: String,
    pub(super) new_speaker: bool,
}

#[derive(Deserialize)]
struct Json3Response {
    events: Option<Vec<Json3Event>>,
}

#[derive(Deserialize)]
struct Json3Event {
    #[serde(rename = "tStartMs")]
    start_ms: Option<i64>,
    #[serde(rename = "dDurationMs")]
    duration_ms: Option<i64>,
    #[serde(rename = "aAppendMs")]
    append_ms: Option<i64>,
    segs: Option<Vec<Json3Seg>>,
}

#[derive(Deserialize)]
struct Json3Seg {
    utf8: Option<String>,
}

pub(super) async fn fetch_transcript(
    http: &ind_egress::GuardedHttpClient,
    base_url: &str,
) -> Vec<TranscriptSegment> {
    if let Ok(json3_url) = prepare_timedtext_url(base_url, "json3", false)
        && let Some(body) = fetch_text(http, &json3_url).await
        && let Some(segs) = parse_json3(&body)
        && !segs.is_empty()
    {
        return segs;
    }

    if let Ok(asr_url) = prepare_timedtext_url(base_url, "json3", true)
        && let Some(body) = fetch_text(http, &asr_url).await
        && let Some(segs) = parse_json3(&body)
        && !segs.is_empty()
    {
        return segs;
    }

    if let Ok(xml_url) = prepare_timedtext_url(base_url, "xml", false)
        && let Some(body) = fetch_text(http, &xml_url).await
    {
        return parse_xml(&body);
    }

    Vec::new()
}

fn prepare_timedtext_url(raw: &str, fmt: &str, with_asr: bool) -> Result<String, ()> {
    let mut url = url::Url::parse(raw).map_err(|_| ())?;
    {
        let mut pairs: Vec<(String, String)> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .filter(|(k, _)| k != "fmt" && k != "xosf" && (with_asr || k != "kind"))
            .collect();
        pairs.push(("fmt".into(), fmt.into()));
        if with_asr {
            pairs.retain(|(k, _)| k != "kind");
            pairs.push(("kind".into(), "asr".into()));
        }
        let mut q = url.query_pairs_mut();
        q.clear();
        for (k, v) in pairs {
            q.append_pair(&k, &v);
        }
    }
    Ok(url.into())
}

async fn fetch_text(http: &ind_egress::GuardedHttpClient, url: &str) -> Option<String> {
    // A guard rejection (private/internal target) resolves to None, so ingest degrades to an
    // empty transcript rather than reaching a blocked host.
    let resp = http.get(url).ok()?.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.text().await.ok()
}

pub(super) fn parse_json3(body: &str) -> Option<Vec<TranscriptSegment>> {
    let data: Json3Response = serde_json::from_str(body).ok()?;
    let events = data.events?;
    let mut segments = Vec::new();
    for event in events {
        if event.append_ms.is_some() {
            continue;
        }
        let Some(segs) = event.segs else {
            continue;
        };
        let raw = segs
            .into_iter()
            .filter_map(|s| s.utf8)
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let raw = raw.trim().to_string();
        if raw.is_empty() {
            continue;
        }
        let start = event.start_ms.unwrap_or(0);
        let end = event.duration_ms.map(|d| start + d);
        segments.push(normalize_segment(&raw, start, end));
    }
    Some(segments)
}

pub(super) fn parse_xml(body: &str) -> Vec<TranscriptSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while let Some(open_idx) = body[cursor..].find("<text") {
        let abs_open = cursor + open_idx;
        let after_tag_start = abs_open + 5;
        let Some(gt_offset) = body[after_tag_start..].find('>') else {
            break;
        };
        let attrs_end = after_tag_start + gt_offset;
        let attrs = &body[after_tag_start..attrs_end];
        let content_start = attrs_end + 1;
        let Some(close_offset) = body[content_start..].find("</text>") else {
            break;
        };
        let content_end = content_start + close_offset;
        let raw_inner = &body[content_start..content_end];
        cursor = content_end + 7;

        let start_ms = parse_attr(attrs, "start")
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| (f * 1000.0).round() as i64)
            .unwrap_or(0);
        let dur_ms = parse_attr(attrs, "dur")
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| (f * 1000.0).round() as i64);
        let end_ms = dur_ms.map(|d| start_ms + d);
        let decoded = decode_xml_entities(raw_inner);
        let trimmed = decoded.trim();
        if trimmed.is_empty() {
            continue;
        }
        segments.push(normalize_segment(trimmed, start_ms, end_ms));
    }
    segments
}

fn parse_attr(attrs: &str, name: &str) -> Option<String> {
    let key = format!("{}=\"", name);
    let start = attrs.find(&key)? + key.len();
    let end = attrs[start..].find('"')? + start;
    Some(attrs[start..end].to_string())
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn normalize_segment(raw: &str, start_ms: i64, end_ms: Option<i64>) -> TranscriptSegment {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix(">>") {
        let after = rest.trim();
        if !after.is_empty() {
            return TranscriptSegment {
                start_ms,
                end_ms,
                text: after.to_string(),
                new_speaker: true,
            };
        }
    }
    TranscriptSegment {
        start_ms,
        end_ms,
        text: trimmed.to_string(),
        new_speaker: false,
    }
}

pub(super) fn split_on_speaker_markers(segments: Vec<TranscriptSegment>) -> Vec<TranscriptSegment> {
    let mut result = Vec::new();
    for seg in segments {
        let parts: Vec<&str> = seg.text.split(">>").collect();
        if parts.len() <= 1 {
            result.push(seg);
            continue;
        }
        for (i, part) in parts.iter().enumerate() {
            let text = part.trim();
            if text.is_empty() {
                continue;
            }
            result.push(TranscriptSegment {
                start_ms: seg.start_ms,
                end_ms: seg.end_ms,
                text: text.to_string(),
                new_speaker: i > 0,
            });
        }
    }
    result
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn ends_sentence(text: &str) -> bool {
    let trimmed = text.trim_end_matches(['\'', '"', ')', ']']);
    matches!(trimmed.chars().last(), Some('.' | '!' | '?'))
}

fn ends_clause(text: &str) -> bool {
    let trimmed = text.trim_end_matches(['\'', '"', ')', ']']);
    matches!(trimmed.chars().last(), Some(',' | ';' | ':'))
}

pub(super) fn paragraphize(segments: Vec<TranscriptSegment>) -> Vec<Vec<TranscriptSegment>> {
    const SILENCE_GAP_MS: i64 = 4000;
    const TARGET_WORDS: usize = 80;
    const MAX_WORDS: usize = 150;

    if segments.is_empty() {
        return Vec::new();
    }

    let mut result: Vec<Vec<TranscriptSegment>> = Vec::new();
    let mut current: Vec<TranscriptSegment> = Vec::new();
    let mut current_words: usize = 0;
    let mut sentence_idx: Option<usize> = None;
    let mut clause_idx: Option<usize> = None;

    let mut prev_end_ms: Option<i64> = None;
    let mut prev_start_ms: Option<i64> = None;

    let scan = |buf: &[TranscriptSegment]| -> (Option<usize>, Option<usize>) {
        let mut s = None;
        let mut c = None;
        for (j, seg) in buf.iter().enumerate() {
            if ends_sentence(&seg.text) {
                s = Some(j);
                c = Some(j);
            } else if ends_clause(&seg.text) {
                c = Some(j);
            }
        }
        (s, c)
    };

    for seg in segments.into_iter() {
        let true_gap = match (prev_end_ms, prev_start_ms) {
            (Some(end), _) => seg.start_ms - end,
            (None, Some(start)) => seg.start_ms - start,
            _ => 0,
        };

        if !current.is_empty() && (true_gap > SILENCE_GAP_MS || seg.new_speaker) {
            result.push(std::mem::take(&mut current));
            current_words = 0;
            sentence_idx = None;
            clause_idx = None;
        }

        prev_end_ms = seg.end_ms;
        prev_start_ms = Some(seg.start_ms);

        let words = count_words(&seg.text);
        let ended_sentence = ends_sentence(&seg.text);
        let ended_clause = ends_clause(&seg.text);
        current.push(seg);
        current_words += words;
        let last_idx = current.len() - 1;
        if ended_sentence {
            sentence_idx = Some(last_idx);
            clause_idx = Some(last_idx);
        } else if ended_clause {
            clause_idx = Some(last_idx);
        }

        if current_words >= TARGET_WORDS
            && let Some(idx) = sentence_idx
        {
            let tail: Vec<TranscriptSegment> = current.drain(idx + 1..).collect();
            let para = std::mem::replace(&mut current, tail);
            result.push(para);
            current_words = current.iter().map(|s| count_words(&s.text)).sum();
            let (s, c) = scan(&current);
            sentence_idx = s;
            clause_idx = c;
            continue;
        }

        if current_words >= MAX_WORDS {
            let break_idx = sentence_idx.or(clause_idx).unwrap_or(current.len() - 1);
            let tail: Vec<TranscriptSegment> = current.drain(break_idx + 1..).collect();
            let para = std::mem::replace(&mut current, tail);
            result.push(para);
            current_words = current.iter().map(|s| count_words(&s.text)).sum();
            let (s, c) = scan(&current);
            sentence_idx = s;
            clause_idx = c;
        }
    }

    if !current.is_empty() {
        result.push(current);
    }
    result
}
