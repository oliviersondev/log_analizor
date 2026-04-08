use std::time::{SystemTime, UNIX_EPOCH};

pub struct SampleLog {
    pub name: &'static str,
    pub raw: &'static str,
}

static SAMPLE_LOGS: &[SampleLog] = &[
    SampleLog {
        name: "json_db_timeout",
        raw: r#"{
  "level": "ERROR",
  "service": "invoice-sync",
  "message": "Database connection timeout while syncing invoice #48291",
  "timestamp": "2026-04-05T10:12:34Z",
  "error_code": "DB_TIMEOUT",
  "response_time_ms": 3120
}"#,
    },
    SampleLog {
        name: "json_auth_invalid_token",
        raw: r#"{
  "level": "WARN",
  "service": "api-gateway",
  "message": "Invalid JWT signature from mobile client",
  "timestamp": "2026-04-05T11:02:03Z",
  "error_code": "AUTH_INVALID_TOKEN",
  "response_time_ms": 220
}"#,
    },
    SampleLog {
        name: "json_upstream_502",
        raw: r#"{
  "level": "ERROR",
  "service": "edge-proxy",
  "message": "Upstream returned 502 for /billing/export",
  "timestamp": "2026-04-05T12:22:00Z",
  "error_code": "UPSTREAM_502",
  "response_time_ms": 780
}"#,
    },
    SampleLog {
        name: "cloudfront_502",
        raw: "2026-04-08 09:10:11 CDG3 123 1.2.3.4 GET d111111abcdef8.cloudfront.net /api 502 - Mozilla/5.0 - - Error abc 0.123",
    },
    SampleLog {
        name: "cloudfront_404",
        raw: "2026-04-08 09:14:49 CDG3 98 4.3.2.1 GET d111111abcdef8.cloudfront.net /assets/missing.js 404 - Mozilla/5.0 - - Miss xyz 0.041",
    },
    SampleLog {
        name: "syslog_failed_auth",
        raw: "Apr 08 12:34:56 prod-host sshd[1234]: Failed password for invalid user admin from 10.0.0.1",
    },
    SampleLog {
        name: "syslog_kernel_panic",
        raw: "Apr 08 13:02:11 node-1 kernel[987]: kernel panic - not syncing: Fatal exception",
    },
    SampleLog {
        name: "plain_timeout",
        raw: "checkout-service timeout after 3200ms while calling db-primary",
    },
    SampleLog {
        name: "plain_invalid_jwt",
        raw: "auth middleware rejected request: invalid token provided by client",
    },
    SampleLog {
        name: "plain_upstream_502",
        raw: "gateway received HTTP 502 from upstream payment-service",
    },
];

pub fn pick_random_sample() -> &'static SampleLog {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);

    let index = seed % SAMPLE_LOGS.len();
    &SAMPLE_LOGS[index]
}
