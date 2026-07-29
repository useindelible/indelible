use super::transcript::{TranscriptSegment, paragraphize, split_on_speaker_markers};

pub(super) struct BuildReaderHtmlInput<'a> {
    pub(super) video_id: &'a str,
    pub(super) description: &'a str,
    pub(super) channel_name: &'a str,
    pub(super) view_count: Option<&'a str>,
    pub(super) duration_seconds: Option<i32>,
    pub(super) segments: &'a [TranscriptSegment],
}

pub(super) fn build_reader_html(opts: BuildReaderHtmlInput<'_>) -> String {
    let escaped_id = escape_html(opts.video_id);
    let description_html = escape_html(opts.description).replace('\n', "<br>");
    let channel_initial = opts
        .channel_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .map(|s| escape_html(&s))
        .unwrap_or_else(|| "?".to_string());
    let channel_name_html = if opts.channel_name.is_empty() {
        escape_html("Unknown channel")
    } else {
        escape_html(opts.channel_name)
    };

    let mut stat_parts: Vec<String> = Vec::new();
    if let Some(vc) = opts.view_count {
        stat_parts.push(escape_html(&format_view_count(vc)));
    }
    if let Some(d) = opts.duration_seconds.filter(|d| *d > 0) {
        stat_parts.push(escape_html(&format_duration_human(d)));
    }
    let stats_html = stat_parts.join("<span class=\"yt-stat-dot\"></span>");

    let mut html = format!(
        "<div class=\"yt-embed\">\n  <iframe width=\"560\" height=\"315\" src=\"https://www.youtube.com/embed/{escaped_id}\" frameborder=\"0\" allowfullscreen></iframe>\n</div>\n<div class=\"yt-channel-header\">\n  <div class=\"yt-channel-avatar\">{channel_initial}</div>\n  <div class=\"yt-channel-info\">\n    <span class=\"yt-channel-name\">{channel_name_html}</span>\n    <div class=\"yt-video-stats\">{stats_html}</div>\n  </div>\n</div>\n<div class=\"yt-description\">{description_html}</div>"
    );

    if !opts.segments.is_empty() {
        let normalized = split_on_speaker_markers(opts.segments.to_vec());
        let paragraphs = paragraphize(normalized);
        let paragraphs_html = paragraphs
            .into_iter()
            .map(|para| {
                let spans = para
                    .into_iter()
                    .map(|seg| {
                        format!(
                            "<span class=\"t-seg\" data-t=\"{}\">{}</span>",
                            escape_html(&format_timestamp(seg.start_ms)),
                            escape_html(&seg.text)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("<p>{}</p>", spans)
            })
            .collect::<Vec<_>>()
            .join("\n");

        html.push_str("\n<section class=\"yt-transcript\">\n  <h2>Transcript</h2>\n  <div class=\"transcript-flow\">");
        html.push_str(&paragraphs_html);
        html.push_str("</div>\n</section>");
    }

    html
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub(super) fn format_view_count(raw: &str) -> String {
    let Ok(n) = raw.parse::<u64>() else {
        return raw.to_string();
    };
    if n >= 1_000_000_000 {
        format!("{}B views", trim_zero_decimal(n as f64 / 1_000_000_000.0))
    } else if n >= 1_000_000 {
        format!("{}M views", trim_zero_decimal(n as f64 / 1_000_000.0))
    } else if n >= 1_000 {
        format!("{}K views", trim_zero_decimal(n as f64 / 1_000.0))
    } else {
        format!("{} views", n)
    }
}

fn trim_zero_decimal(n: f64) -> String {
    let formatted = format!("{:.1}", n);
    if let Some(stripped) = formatted.strip_suffix(".0") {
        stripped.to_string()
    } else {
        formatted
    }
}

pub(super) fn format_duration_human(seconds: i32) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

fn format_timestamp(ms: i64) -> String {
    let total_seconds = (ms / 1000).max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
