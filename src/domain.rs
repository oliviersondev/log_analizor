use serde::{Deserialize, Serialize};

pub struct Context7Query {
    pub library: &'static str,
    pub framework: &'static str,
    pub topic: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppLog {
    level: String,
    service: String,
    message: String,
    timestamp: String,
    error_code: Option<String>,
    response_time_ms: Option<u64>,
}

pub fn parse_log(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            format!(
                "Parsed log => level={}, service={}, message={}, error_code={:?}, response_time_ms={:?}, timestamp={}",
                log.level,
                log.service,
                log.message,
                log.error_code,
                log.response_time_ms,
                log.timestamp
            )
        }
        Err(err) => format!("Parse error: {err}"),
    }
}

pub fn classify_incident(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            let severity = if log.level.eq_ignore_ascii_case("error") {
                "high"
            } else if log.response_time_ms.unwrap_or(0) > 2000 {
                "medium"
            } else {
                "low"
            };

            format!(
                "Incident classification => severity={}, service={}, probable_cause={}",
                severity,
                log.service,
                infer_cause(&log)
            )
        }
        Err(err) => format!("Classification impossible: {err}"),
    }
}

pub fn suggest_fix(raw_log: String) -> String {
    match serde_json::from_str::<AppLog>(&raw_log) {
        Ok(log) => {
            let suggestion = if let Some(code) = &log.error_code {
                match code.as_str() {
                    "DB_TIMEOUT" => {// TODO c'est une kley d'enum utilisé partout
                        "Verifier la latence DB, le pool de connexions et les requetes lentes."
                    }
                    "AUTH_INVALID_TOKEN" => {
                        "Controler la signature/expiration du token et les logs d'authentification."
                    }
                    "UPSTREAM_502" => {
                        "Verifier la disponibilite du service upstream et ajouter retry/circuit breaker."
                    }
                    _ => "Analyser les logs correles et verifier les metriques du service.",
                }
            } else if log.response_time_ms.unwrap_or(0) > 2000 {
                "Inspecter les dependances lentes et la saturation CPU / I/O."
            } else {
                "Aucune action critique immediate, surveiller l'evolution."
            };

            format!("Suggested action => {suggestion}")
        }
        Err(err) => format!("Suggestion impossible: {err}"),
    }
}

pub fn infer_cause(log: &AppLog) -> &'static str {
    if let Some(code) = &log.error_code {
        match code.as_str() {
            "DB_TIMEOUT" => "database latency or pool exhaustion",
            "AUTH_INVALID_TOKEN" => "authentication issue",
            "UPSTREAM_502" => "upstream service instability",
            _ => "unknown application error",
        }
    } else if log.response_time_ms.unwrap_or(0) > 2000 {
        "performance degradation"
    } else {
        "minor event"
    }
}

pub fn context7_query_from_raw_log(raw_log: &str) -> Option<Context7Query> {
    let log = serde_json::from_str::<AppLog>(raw_log).ok()?;

    match log.error_code.as_deref() {
        Some("DB_TIMEOUT") => Some(Context7Query {
            library: "postgresql", // TODO c'est pas forcement cette techno
            framework: "postgresql",
            topic: "connection timeout",
        }),
        Some("AUTH_INVALID_TOKEN") => Some(Context7Query {
            library: "auth0", // TODO c'est pas forcement cette techno
            framework: "docs",
            topic: "jwt validation",
        }),
        Some("UPSTREAM_502") => Some(Context7Query {
            library: "nginx", // TODO c'est pas forcement cette techno
            framework: "nginx",
            topic: "502 bad gateway",
        }),
        _ => None,
    }
}
