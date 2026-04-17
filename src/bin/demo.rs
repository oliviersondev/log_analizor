use log_analizor::analyzer::{Analyzer, analysis_prompt};
use log_analizor::app::runner;
use log_analizor::sample_logs::pick_random_sample;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = Analyzer::from_env()?;
    let sample = pick_random_sample();

    let prompt = analysis_prompt(sample.raw);
    println!("Cas de test: {}\n{}", sample.name, prompt);

    println!("Agent response :");

    let mut stdout = std::io::stdout();
    runner::run_raw_log_to_writer(&analyzer, sample.raw.to_string(), &mut stdout).await?;
    stdout.flush()?;

    Ok(())
}
