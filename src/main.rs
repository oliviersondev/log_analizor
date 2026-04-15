use log_analizor::analyzer::{AnalysisEvent, analyze_raw_log_stream};
use std::io::{IsTerminal, Read, Write};

const USAGE: &str = "Usage:\n  cargo run --bin log_analizor -- --log \"<log brut>\"\n  cat /path/to/log.txt | cargo run --bin log_analizor";

fn parse_raw_log_input() -> Result<String, std::io::Error> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        std::process::exit(0);
    }

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--log" | "-l" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Missing value for {}\n\n{USAGE}", args[i]),
                    )
                })?;

                if value.trim().is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Log input is empty\n\n{USAGE}"),
                    ));
                }

                return Ok(value.to_string());
            }
            flag if flag.starts_with('-') => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown argument: {flag}\n\n{USAGE}"),
                ));
            }
            _ => {}
        }
        i += 1;
    }

    if std::io::stdin().is_terminal() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("No log input provided\n\n{USAGE}"),
        ));
    }

    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;

    if buffer.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Stdin log input is empty\n\n{USAGE}"),
        ));
    }

    Ok(buffer)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_log = parse_raw_log_input()?;

    println!("Agent response :");

    analyze_raw_log_stream(raw_log, |event| match event {
        AnalysisEvent::TextDelta(text) => {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
        AnalysisEvent::Done { usage_line } => {
            println!("\n\n{usage_line}");
        }
    })
    .await?;

    Ok(())
}
