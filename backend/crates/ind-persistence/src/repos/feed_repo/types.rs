use ind_application::AppError;
use ind_domain::{DomainError, FeedSearchSurface, FeedStatus, FeedType, FeedVisibility};

pub(super) fn map_source_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("feed_source", "feed source conflict", err)
}

pub(super) fn map_subscription_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("feed_subscription", "feed subscription conflict", err)
}

pub(super) fn map_entry_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error("feed_source_entry", "feed source entry conflict", err)
}

pub(super) fn map_provider_error(err: sqlx::Error) -> AppError {
    super::super::map_sqlx_error(
        "FeedProviderInstance",
        "feed provider instance conflict",
        err,
    )
}

pub(super) fn parse_feed_type(s: &str) -> Result<FeedType, AppError> {
    match s {
        "rss" => Ok(FeedType::Rss),
        "atom" => Ok(FeedType::Atom),
        "podcast" => Ok(FeedType::Podcast),
        "youtube" => Ok(FeedType::Youtube),
        "twitter" => Ok(FeedType::Twitter),
        "newsletter" => Ok(FeedType::Newsletter),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown feed_type: {other}"),
        })),
    }
}

pub(super) fn feed_type_to_str(ft: FeedType) -> &'static str {
    match ft {
        FeedType::Rss => "rss",
        FeedType::Atom => "atom",
        FeedType::Podcast => "podcast",
        FeedType::Youtube => "youtube",
        FeedType::Twitter => "twitter",
        FeedType::Newsletter => "newsletter",
    }
}

pub(super) fn parse_feed_status(s: &str) -> Result<FeedStatus, AppError> {
    s.parse::<FeedStatus>().map_err(|_| {
        AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown feed_status: {s}"),
        })
    })
}

pub(super) fn feed_status_to_str(fs: FeedStatus) -> &'static str {
    fs.as_str()
}

pub(super) fn parse_visibility(s: &str) -> Result<FeedVisibility, AppError> {
    match s {
        "public" => Ok(FeedVisibility::Public),
        "private" => Ok(FeedVisibility::Private),
        other => Err(AppError::Domain(DomainError::InvariantViolation {
            message: format!("unknown feed visibility: {other}"),
        })),
    }
}

pub(super) fn visibility_to_str(v: FeedVisibility) -> &'static str {
    match v {
        FeedVisibility::Public => "public",
        FeedVisibility::Private => "private",
    }
}

pub(super) fn surface_to_str(surface: FeedSearchSurface) -> &'static str {
    match surface {
        FeedSearchSurface::All => "all",
        FeedSearchSurface::Rss => "rss",
        FeedSearchSurface::Youtube => "youtube",
        FeedSearchSurface::Twitter => "twitter",
    }
}

pub(super) fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
