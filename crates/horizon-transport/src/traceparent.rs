//! OpenTelemetry W3C `traceparent` propagation for gRPC metadata.

use tonic::metadata::MetadataMap;

/// Parsed W3C trace context from a `traceparent` header value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub raw: String,
    pub trace_id: String,
    pub parent_span_id: String,
    pub sampled: bool,
}

/// Extract and parse the W3C `traceparent` header from gRPC metadata.
pub fn extract_traceparent(metadata: &MetadataMap) -> Option<TraceContext> {
    let raw = metadata
        .get("traceparent")
        .or_else(|| metadata.get("Traceparent"))
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    parse_traceparent(raw)
}

/// Create a tracing span with OpenTelemetry-compatible fields from gRPC metadata.
pub fn span_from_metadata(metadata: &MetadataMap, operation: &'static str) -> tracing::Span {
    match extract_traceparent(metadata) {
        Some(ctx) => tracing::info_span!(
            operation,
            otel.trace_id = %ctx.trace_id,
            otel.span_id = %ctx.parent_span_id,
            otel.sampled = ctx.sampled,
            w3c.traceparent = %ctx.raw,
        ),
        None => tracing::info_span!(operation),
    }
}

fn parse_traceparent(raw: &str) -> Option<TraceContext> {
    let mut parts = raw.split('-');
    let version = parts.next()?;
    let trace_id = parts.next()?;
    let parent_span_id = parts.next()?;
    let flags = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    if version != "00" || trace_id.len() != 32 || parent_span_id.len() != 16 || flags.len() != 2 {
        return None;
    }

    if !trace_id.chars().all(|c| c.is_ascii_hexdigit())
        || !parent_span_id.chars().all(|c| c.is_ascii_hexdigit())
        || !flags.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }

    let sampled = flags.ends_with('1');

    Some(TraceContext {
        raw: raw.to_owned(),
        trace_id: trace_id.to_ascii_lowercase(),
        parent_span_id: parent_span_id.to_ascii_lowercase(),
        sampled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

    #[test]
    fn parses_valid_traceparent() {
        let ctx = parse_traceparent(
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
        )
        .expect("valid traceparent");

        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.parent_span_id, "b7ad6b7169203331");
        assert!(ctx.sampled);
    }

    #[test]
    fn extracts_from_metadata_map() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "traceparent",
            MetadataValue::from_static(
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            ),
        );

        let ctx = extract_traceparent(&metadata).expect("extracted");
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
    }
}
