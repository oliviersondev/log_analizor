use std::ffi::OsString;
use std::io::{IsTerminal, Read};

use clap::Parser;
use clap::error::ErrorKind;

const DEFAULT_MAX_LOG_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInput {
    pub raw_log: Option<String>,
}

#[derive(Debug, Parser)]
#[command(
    name = "log_analizor",
    about = "Analyse des logs en streaming via Ollama",
    long_about = None
)]
struct CliArgs {
    #[arg(short = 'l', long = "log")]
    raw_log: Option<String>,
}

fn max_log_bytes_from_env() -> usize {
    std::env::var("MAX_LOG_BYTES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_LOG_BYTES)
}

pub fn parse_cli_input_from<I, T>(args: I) -> Result<CliInput, std::io::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let parsed = CliArgs::try_parse_from(args).map_err(|err| {
        if matches!(
            err.kind(),
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
        ) {
            let _ = err.print();
            std::process::exit(0);
        }

        std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
    })?;

    Ok(CliInput {
        raw_log: parsed.raw_log,
    })
}

pub fn resolve_raw_log_input<R: Read>(
    cli_input: CliInput,
    stdin: R,
    stdin_is_terminal: bool,
    max_log_bytes: usize,
) -> Result<String, std::io::Error> {
    if let Some(raw_log) = cli_input.raw_log {
        if raw_log.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Log input is empty",
            ));
        }
        return Ok(raw_log);
    }

    if stdin_is_terminal {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No log input provided. Use --log '<log brut>' or pipe stdin.",
        ));
    }

    let mut buffer = String::new();
    let read_limit = max_log_bytes
        .checked_add(1)
        .ok_or_else(|| std::io::Error::other("MAX_LOG_BYTES value is too large"))?;

    stdin.take(read_limit as u64).read_to_string(&mut buffer)?;

    if buffer.len() > max_log_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Stdin log input is too large (max {} bytes). Set MAX_LOG_BYTES to override.",
                max_log_bytes
            ),
        ));
    }

    if buffer.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Stdin log input is empty",
        ));
    }

    Ok(buffer)
}

pub fn read_raw_log_from_env_args() -> Result<String, std::io::Error> {
    let cli_input = parse_cli_input_from(std::env::args_os())?;
    let max_log_bytes = max_log_bytes_from_env();
    let stdin = std::io::stdin();

    resolve_raw_log_input(cli_input, stdin.lock(), stdin.is_terminal(), max_log_bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{CliInput, parse_cli_input_from, resolve_raw_log_input};

    #[test]
    fn parses_log_argument() {
        let args = ["log_analizor", "--log", "hello"];
        let parsed = parse_cli_input_from(args).expect("args should parse");

        assert_eq!(
            parsed,
            CliInput {
                raw_log: Some("hello".to_string())
            }
        );
    }

    #[test]
    fn returns_error_on_unknown_flag() {
        let args = ["log_analizor", "--unknown"];
        let err = parse_cli_input_from(args).expect_err("unknown flag should fail");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn reads_from_stdin_when_no_log_argument() {
        let input = CliInput { raw_log: None };
        let stdin = Cursor::new("line from stdin\n");

        let raw = resolve_raw_log_input(input, stdin, false, 1_048_576)
            .expect("stdin content should be accepted");
        assert_eq!(raw, "line from stdin\n");
    }

    #[test]
    fn returns_error_when_no_input_and_terminal() {
        let input = CliInput { raw_log: None };
        let stdin = Cursor::new("");

        let err = resolve_raw_log_input(input, stdin, true, 1_048_576)
            .expect_err("terminal without input should fail");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
