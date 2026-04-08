use super::{classify_incident, parse_log, prompt_header_for_raw_log};

#[test]
fn parses_json_log() {
    let raw = r#"{"level":"ERROR","service":"billing","message":"db timeout","timestamp":"2026-01-01T10:00:00Z","error_code":"DB_TIMEOUT","response_time_ms":3200}"#;
    let out = parse_log(raw.to_string());
    assert!(out.contains("format=json"));
    assert!(out.contains("service=billing"));
}

#[test]
fn parses_cloudfront_log() {
    let raw = "2026-04-08 09:10:11 CDG3 123 1.2.3.4 GET d111111abcdef8.cloudfront.net /api 502 - Mozilla/5.0 - - Error abc 0.123";
    let out = parse_log(raw.to_string());
    assert!(out.contains("format=cloudfront"));
    assert!(out.contains("error_code=Some(\"UPSTREAM_502\")"));
}

#[test]
fn classifies_syslog_as_high_on_failure_keywords() {
    let raw = "Apr 08 12:34:56 prod-host sshd[1234]: Failed password for invalid user admin from 10.0.0.1";
    let out = classify_incident(raw.to_string());
    assert!(out.contains("severity=high"));
}

#[test]
fn prompt_header_matches_cloudfront_format() {
    let raw = "2026-04-08 09:10:11 CDG3 123 1.2.3.4 GET d111111abcdef8.cloudfront.net /api 502 - Mozilla/5.0 - - Error abc 0.123";
    let header = prompt_header_for_raw_log(raw);
    assert!(header.contains("CloudFront"));
}
