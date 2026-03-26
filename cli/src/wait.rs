use anyhow::{bail, Context, Result};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

use crate::types::{WaitEvent, WaitResult};

/// Connect to the SSE stream and wait for the notification to be resolved.
///
/// Returns `WaitResult::Action(id)` if an action was clicked,
/// `WaitResult::Dismissed` if the notification was dismissed,
/// or `WaitResult::Timeout` if `timeout_secs` expires.
pub async fn wait_for_resolution(
    base_url: &str,
    notification_id: &str,
    timeout_secs: u64,
) -> Result<WaitResult> {
    let url = format!("{base_url}/notify/{notification_id}/wait");

    let client = reqwest::Client::builder()
        // No HTTP timeout — we rely on our own tokio::time::timeout
        .timeout(Duration::from_secs(timeout_secs + 10))
        .build()
        .context("failed to build HTTP client for SSE")?;

    let result = timeout(
        Duration::from_secs(timeout_secs),
        read_sse_stream(&client, &url),
    )
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Ok(WaitResult::Timeout),
    }
}

/// Read the SSE stream line by line and parse events.
async fn read_sse_stream(client: &reqwest::Client, url: &str) -> Result<WaitResult> {
    let response = client
        .get(url)
        .header("Accept", "text/event-stream")
        .send()
        .await
        .context("failed to connect SSE stream — is syncfu running?")?;

    if !response.status().is_success() {
        bail!(
            "SSE endpoint returned {} — notification may not exist",
            response.status()
        );
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("SSE stream read error")?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete lines from the buffer
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end_matches('\r').to_string();
            buffer = buffer[pos + 1..].to_string();

            if let Some(result) = parse_sse_line(&line) {
                return Ok(result);
            }
        }
    }

    // Stream closed without a resolution event
    Ok(WaitResult::Dismissed)
}

/// Parse a single SSE line. Returns Some(WaitResult) if this is a resolution event.
/// Returns None for connected events, comments, empty lines, etc.
fn parse_sse_line(line: &str) -> Option<WaitResult> {
    // SSE data lines start with "data:"
    let data = if let Some(stripped) = line.strip_prefix("data:") {
        stripped.trim()
    } else {
        // Ignore event:, id:, retry:, comments (:), and empty lines
        return None;
    };

    if data.is_empty() {
        return None;
    }

    let event: WaitEvent = match serde_json::from_str(data) {
        Ok(e) => e,
        Err(_) => return None, // Skip unparseable data
    };

    match event {
        WaitEvent::Connected => None, // Not a resolution, keep waiting
        WaitEvent::Action { action_id } => Some(WaitResult::Action(action_id)),
        WaitEvent::Dismissed => Some(WaitResult::Dismissed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_action() {
        let line = r#"data: {"event":"action","action_id":"approve"}"#;
        let result = parse_sse_line(line);
        assert_eq!(result, Some(WaitResult::Action("approve".to_string())));
    }

    #[test]
    fn test_parse_sse_line_dismissed() {
        let line = r#"data: {"event":"dismissed"}"#;
        let result = parse_sse_line(line);
        assert_eq!(result, Some(WaitResult::Dismissed));
    }

    #[test]
    fn test_parse_sse_line_connected_returns_none() {
        let line = r#"data: {"event":"connected"}"#;
        let result = parse_sse_line(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_sse_line_empty_data() {
        assert_eq!(parse_sse_line("data:"), None);
        assert_eq!(parse_sse_line("data: "), None);
    }

    #[test]
    fn test_parse_sse_line_comment() {
        assert_eq!(parse_sse_line(": comment"), None);
    }

    #[test]
    fn test_parse_sse_line_event_prefix_ignored() {
        assert_eq!(parse_sse_line("event: message"), None);
    }

    #[test]
    fn test_parse_sse_line_empty_line() {
        assert_eq!(parse_sse_line(""), None);
    }

    #[test]
    fn test_parse_sse_line_malformed_json() {
        assert_eq!(parse_sse_line("data: {not json}"), None);
    }

    #[test]
    fn test_parse_sse_line_no_data_prefix() {
        assert_eq!(parse_sse_line("id: 123"), None);
    }

    #[test]
    fn test_parse_sse_line_data_with_no_space() {
        let line = r#"data:{"event":"dismissed"}"#;
        let result = parse_sse_line(line);
        assert_eq!(result, Some(WaitResult::Dismissed));
    }
}
