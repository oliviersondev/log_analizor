mod normalize;

use normalize::{LogFormat, NormalizedLog, normalize_log};

const ERROR_DB_TIMEOUT: &str = "DB_TIMEOUT";
const ERROR_AUTH_INVALID_TOKEN: &str = "AUTH_INVALID_TOKEN";
const ERROR_UPSTREAM_502: &str = "UPSTREAM_502";

pub struct Context7Query {
    pub library: &'static str,
    pub framework: &'static str,
    pub topic: &'static str,
}

pub fn parse_log(raw_log: String) -> String {
    let log = normalize_log(&raw_log);

    format!(
        "Parsed log => format={}, level={}, service={}, message={}, error_code={:?}, response_time_ms={:?}, timestamp={}",
        log.format.as_str(),
        log.level,
        log.service,
        log.message,
        log.error_code,
        log.response_time_ms,
        log.timestamp
    )
}

pub fn classify_incident(raw_log: String) -> String {
    let log = normalize_log(&raw_log);

    format!(
        "Incident classification => severity={}, service={}, probable_cause={}",
        infer_severity(&log),
        log.service,
        infer_cause(&log)
    )
}

pub fn suggest_fix(raw_log: String) -> String {
    let log = normalize_log(&raw_log);

    let suggestion = match log.error_code.as_deref() {
        Some(ERROR_DB_TIMEOUT) => {
            "Verifier la latence DB, le pool de connexions et les requetes lentes."
        }
        Some(ERROR_AUTH_INVALID_TOKEN) => {
            "Controler la signature/expiration du token et les logs d'authentification."
        }
        Some(ERROR_UPSTREAM_502) => {
            "Verifier la disponibilite du service upstream et ajouter retry/circuit breaker."
        }
        Some(_) => "Analyser les logs correles et verifier les metriques du service.",
        None if log.status_code == Some(502) => {
            "Verifier la disponibilite du service upstream et ajouter retry/circuit breaker."
        }
        None if log.response_time_ms.unwrap_or(0) > 2000 => {
            "Inspecter les dependances lentes et la saturation CPU / I/O."
        }
        None => "Aucune action critique immediate, surveiller l'evolution.",
    };

    format!("Suggested action => {suggestion}")
}

pub fn context7_query_from_raw_log(raw_log: &str) -> Option<Context7Query> {
    let log = normalize_log(raw_log);

    match log.error_code.as_deref() {
        Some(ERROR_DB_TIMEOUT) => Some(Context7Query {
            library: "postgresql",
            framework: "postgresql",
            topic: "connection timeout",
        }),
        Some(ERROR_AUTH_INVALID_TOKEN) => Some(Context7Query {
            library: "auth0",
            framework: "docs",
            topic: "jwt validation",
        }),
        Some(ERROR_UPSTREAM_502) => Some(Context7Query {
            library: "nginx",
            framework: "nginx",
            topic: "502 bad gateway",
        }),
        _ => None,
    }
}

pub fn prompt_header_for_raw_log(raw_log: &str) -> &'static str {
    match normalize_log(raw_log).format {
        LogFormat::Json => {
            "Analyse ce log JSON applicatif et reponds en francais de maniere structuree:"
        }
        LogFormat::Cloudfront => {
            "Analyse ce log CloudFront et reponds en francais de maniere structuree:"
        }
        LogFormat::Syslog => {
            "Analyse ce log systeme (syslog) et reponds en francais de maniere structuree:"
        }
        LogFormat::PlainText => {
            "Analyse ce log texte libre et reponds en francais de maniere structuree:"
        }
    }
}

fn infer_cause(log: &NormalizedLog) -> &'static str {
    match log.error_code.as_deref() {
        Some(ERROR_DB_TIMEOUT) => "database latency or pool exhaustion",
        Some(ERROR_AUTH_INVALID_TOKEN) => "authentication issue",
        Some(ERROR_UPSTREAM_502) => "upstream service instability",
        Some(_) => "unknown application error",
        None if log.status_code == Some(502) => "upstream service instability",
        None if log.response_time_ms.unwrap_or(0) > 2000 => "performance degradation",
        None => "minor event",
    }
}

fn infer_severity(log: &NormalizedLog) -> &'static str {
    let level = log.level.to_ascii_lowercase();
    let message = log.message.to_ascii_lowercase();

    if matches!(level.as_str(), "error" | "critical" | "fatal")
        || log.status_code.is_some_and(|s| s >= 500)
        || message.contains("panic")
        || message.contains("exception")
        || message.contains("failed")
        || message.contains("timeout")
    {
        "high"
    } else if log.response_time_ms.unwrap_or(0) > 2000
        || log.status_code.is_some_and(|s| (400..500).contains(&s))
    {
        "medium"
    } else {
        "low"
    }
}

#[cfg(test)]
mod tests;
