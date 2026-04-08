use serde::{Deserialize, Serialize};

const ERROR_DB_TIMEOUT: &str = "DB_TIMEOUT";
const ERROR_AUTH_INVALID_TOKEN: &str = "AUTH_INVALID_TOKEN";
const ERROR_UPSTREAM_502: &str = "UPSTREAM_502";

#[derive(Debug, Deserialize, Serialize)]
struct AppLog {
    level: String,
    service: String,
    message: String,
    timestamp: String,
    error_code: Option<String>,
    response_time_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Cloudfront,
    Syslog,
    PlainText,
}

impl LogFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogFormat::Json => "json",
            LogFormat::Cloudfront => "cloudfront",
            LogFormat::Syslog => "syslog",
            LogFormat::PlainText => "plain_text",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedLog {
    pub format: LogFormat,
    pub level: String,
    pub service: String,
    pub message: String,
    pub timestamp: String,
    pub error_code: Option<String>,
    pub response_time_ms: Option<u64>,
    pub status_code: Option<u16>,
}

pub fn normalize_log(raw_log: &str) -> NormalizedLog {
    parse_json(raw_log)
        .or_else(|| parse_cloudfront(raw_log))
        .or_else(|| parse_syslog(raw_log))
        .unwrap_or_else(|| parse_plain_text(raw_log))
}

fn parse_json(raw_log: &str) -> Option<NormalizedLog> {
    let log = serde_json::from_str::<AppLog>(raw_log).ok()?;

    let status_code = extract_status_code(&log.message);
    let error_code = log.error_code.or_else(|| {
        if status_code == Some(502) {
            Some(ERROR_UPSTREAM_502.to_string())
        } else {
            None
        }
    });

    Some(NormalizedLog {
        format: LogFormat::Json,
        level: log.level,
        service: log.service,
        message: log.message,
        timestamp: log.timestamp,
        error_code,
        response_time_ms: log.response_time_ms,
        status_code,
    })
}

fn parse_cloudfront(raw_log: &str) -> Option<NormalizedLog> {
    let line = raw_log
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))?;

    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 9 {
        return None;
    }

    let date = fields[0];
    let time = fields[1];
    if !date.contains('-') || !time.contains(':') {
        return None;
    }

    let method = fields.get(5).copied().unwrap_or("-");
    let host = fields.get(6).copied().unwrap_or("cloudfront");
    let uri = fields.get(7).copied().unwrap_or("-");
    let status_code = fields.get(8).and_then(|s| s.parse::<u16>().ok());
    let client_ip = fields.get(4).copied().unwrap_or("-");

    let response_time_ms = fields
        .iter()
        .rev()
        .find_map(|token| token.parse::<f64>().ok())
        .map(|seconds| (seconds * 1000.0) as u64);

    let level = match status_code {
        Some(code) if code >= 500 => "error",
        Some(code) if code >= 400 => "warn",
        _ => "info",
    }
    .to_string();

    let error_code = if status_code == Some(502) {
        Some(ERROR_UPSTREAM_502.to_string())
    } else {
        None
    };

    Some(NormalizedLog {
        format: LogFormat::Cloudfront,
        level,
        service: host.to_string(),
        message: format!("{method} {uri} status={status_code:?} ip={client_ip}"),
        timestamp: format!("{date}T{time}Z"),
        error_code,
        response_time_ms,
        status_code,
    })
}

fn parse_syslog(raw_log: &str) -> Option<NormalizedLog> {
    let line = raw_log.lines().next()?.trim();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let month = parts[0];
    let day = parts[1];
    let time = parts[2];
    if month.len() != 3 || !time.contains(':') {
        return None;
    }

    let host = parts[3];
    let rest = parts[4..].join(" ");
    let (service_part, message) = rest.split_once(':')?;
    let service = service_part
        .split('[')
        .next()
        .unwrap_or(service_part)
        .trim();
    let message = message.trim();

    let level = infer_level_from_message(message).to_string();
    let status_code = extract_status_code(message);
    let response_time_ms = extract_response_time_ms(message);
    let error_code = infer_error_code_from_message(message, status_code);

    Some(NormalizedLog {
        format: LogFormat::Syslog,
        level,
        service: format!("{host}/{service}"),
        message: message.to_string(),
        timestamp: format!("{month} {day} {time}"),
        error_code,
        response_time_ms,
        status_code,
    })
}

fn parse_plain_text(raw_log: &str) -> NormalizedLog {
    let line = raw_log
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(raw_log);
    let status_code = extract_status_code(line);
    let error_code = infer_error_code_from_message(line, status_code);

    NormalizedLog {
        format: LogFormat::PlainText,
        level: infer_level_from_message(line).to_string(),
        service: "unknown".to_string(),
        message: line.trim().to_string(),
        timestamp: "unknown".to_string(),
        error_code,
        response_time_ms: extract_response_time_ms(line),
        status_code,
    }
}

fn infer_level_from_message(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();

    if lower.contains("error")
        || lower.contains("critical")
        || lower.contains("fatal")
        || lower.contains("panic")
        || lower.contains("exception")
        || lower.contains("failed")
    {
        "error"
    } else if lower.contains("warn") || lower.contains("timeout") {
        "warn"
    } else {
        "info"
    }
}

fn infer_error_code_from_message(message: &str, status_code: Option<u16>) -> Option<String> {
    let lower = message.to_ascii_lowercase();

    if status_code == Some(502) {
        Some(ERROR_UPSTREAM_502.to_string())
    } else if lower.contains("invalid token") || lower.contains("jwt") {
        Some(ERROR_AUTH_INVALID_TOKEN.to_string())
    } else if lower.contains("timeout") {
        Some(ERROR_DB_TIMEOUT.to_string())
    } else {
        None
    }
}

fn extract_status_code(message: &str) -> Option<u16> {
    message.split_whitespace().find_map(|token| {
        let code = token.parse::<u16>().ok()?;
        if (100..=599).contains(&code) {
            Some(code)
        } else {
            None
        }
    })
}

fn extract_response_time_ms(message: &str) -> Option<u64> {
    let compact = message.replace(' ', "");

    if let Some(index) = compact.find("ms") {
        let before = &compact[..index];
        let digits_rev: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        if digits_rev.is_empty() {
            return None;
        }

        let digits: String = digits_rev.chars().rev().collect();
        return digits.parse::<u64>().ok();
    }

    None
}
