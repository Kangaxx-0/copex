//! Parser for Copilot x-quota-snapshot-* response headers.
//!
//! Headers have the format:
//!   x-quota-snapshot-<category>: ent=<int>&ov=<float>&ovPerm=<bool>&rem=<float>&rst=<url-encoded-date>

use http::HeaderMap;
use std::collections::HashMap;

const QUOTA_HEADER_PREFIX: &str = "x-quota-snapshot-";

#[derive(Debug, Clone, PartialEq)]
pub struct CopilotQuotaSnapshot {
    pub entitlement: i64,
    pub overage: f64,
    pub overage_permitted: bool,
    pub percent_remaining: f64,
    pub resets_at: String,
}

pub fn parse_copilot_quota_headers(headers: &HeaderMap) -> HashMap<String, CopilotQuotaSnapshot> {
    let mut quotas = HashMap::new();
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_ascii_lowercase();
        if let Some(category) = header_name.strip_prefix(QUOTA_HEADER_PREFIX)
            && let Some(snapshot) = value.to_str().ok().and_then(parse_quota_snapshot_header)
        {
            quotas.insert(category.to_string(), snapshot);
        }
    }
    quotas
}

pub fn parse_quota_snapshot_header(raw: &str) -> Option<CopilotQuotaSnapshot> {
    let params: HashMap<&str, &str> = raw
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .collect();

    Some(CopilotQuotaSnapshot {
        entitlement: params.get("ent")?.parse().ok()?,
        overage: params.get("ov")?.parse().ok()?,
        overage_permitted: *params.get("ovPerm")? == "true",
        percent_remaining: params.get("rem")?.parse().ok()?,
        resets_at: urlencoding::decode(params.get("rst")?).ok()?.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_parse_quota_snapshot_header() {
        let raw = "ent=-1&ov=0.0&ovPerm=true&rem=100.0&rst=2026-04-01T00%3A00%3A00Z";
        let quota = parse_quota_snapshot_header(raw).unwrap();
        assert_eq!(quota.entitlement, -1);
        assert_eq!(quota.percent_remaining, 100.0);
        assert_eq!(quota.resets_at, "2026-04-01T00:00:00Z");
    }

    #[test]
    fn test_parse_quota_snapshot_header_capped_plan() {
        let raw = "ent=1000&ov=0.0&ovPerm=false&rem=62.3&rst=2026-04-01T00%3A00%3A00Z";
        let quota = parse_quota_snapshot_header(raw).unwrap();
        assert_eq!(quota.entitlement, 1000);
        assert_eq!(quota.percent_remaining, 62.3);
        assert!(!quota.overage_permitted);
    }

    #[test]
    fn test_parse_quota_from_response_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-quota-snapshot-premium_interactions",
            HeaderValue::from_static(
                "ent=-1&ov=0.0&ovPerm=true&rem=100.0&rst=2026-04-01T00%3A00%3A00Z",
            ),
        );
        headers.insert(
            "x-quota-snapshot-chat",
            HeaderValue::from_static(
                "ent=500&ov=0.0&ovPerm=false&rem=80.0&rst=2026-04-01T00%3A00%3A00Z",
            ),
        );

        let quotas = parse_copilot_quota_headers(&headers);
        assert_eq!(quotas.len(), 2);
        assert!(quotas.contains_key("premium_interactions"));
        assert!(quotas.contains_key("chat"));
        assert_eq!(quotas["premium_interactions"].percent_remaining, 100.0);
        assert_eq!(quotas["chat"].percent_remaining, 80.0);
    }

    #[test]
    fn test_parse_quota_from_response_headers_empty() {
        let headers = HeaderMap::new();
        let quotas = parse_copilot_quota_headers(&headers);
        assert!(quotas.is_empty());
    }

    #[test]
    fn test_parse_quota_snapshot_header_malformed() {
        let raw = "garbage";
        let result = parse_quota_snapshot_header(raw);
        assert!(result.is_none());
    }
}
