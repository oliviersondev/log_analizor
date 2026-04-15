use log_analizor::analyzer::{AnalysisEvent, analysis_prompt, analyze_raw_log_stream};
use log_analizor::sample_logs::pick_random_sample;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO c'est un test du coup ça
    let sample = pick_random_sample();

    let prompt = analysis_prompt(sample.raw);
    println!("Cas de test: {}\n{}", sample.name, prompt);

    println!("Agent response :");

    analyze_raw_log_stream(sample.raw.to_string(), |event| match event {
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
