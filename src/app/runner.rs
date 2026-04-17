use std::io::Write;

use crate::analyzer::{AnalysisEvent, Analyzer};
use crate::app::error::AppError;
use crate::app::{events, input};

pub async fn run_cli(stdout: &mut dyn Write) -> Result<(), AppError> {
    let raw_log = input::read_raw_log_from_env_args().map_err(AppError::input)?;
    let analyzer = Analyzer::from_env().map_err(AppError::analyze)?;

    stdout
        .write_all(b"Agent response :\n")
        .map_err(AppError::output)?;
    run_raw_log_to_writer(&analyzer, raw_log, stdout).await
}

pub async fn run_raw_log_stream<F>(
    analyzer: &Analyzer,
    raw_log: String,
    on_event: F,
) -> Result<(), AppError>
where
    F: FnMut(AnalysisEvent),
{
    analyzer
        .analyze_raw_log_stream(raw_log, on_event)
        .await
        .map_err(AppError::analyze)
}

pub async fn run_raw_log_to_writer(
    analyzer: &Analyzer,
    raw_log: String,
    writer: &mut dyn Write,
) -> Result<(), AppError> {
    let mut write_error: Option<std::io::Error> = None;

    run_raw_log_stream(analyzer, raw_log, |event| {
        if write_error.is_some() {
            return;
        }

        if let Some(rendered) = events::render_terminal_event(event) {
            if let Err(error) = writer.write_all(rendered.text.as_bytes()) {
                write_error = Some(error);
                return;
            }

            if rendered.flush
                && let Err(error) = writer.flush()
            {
                write_error = Some(error);
            }
        }
    })
    .await?;

    if let Some(error) = write_error {
        return Err(AppError::output(error));
    }

    Ok(())
}
