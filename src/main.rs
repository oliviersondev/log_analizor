use log_analizor::app::runner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    runner::run_cli(&mut stdout).await?;

    Ok(())
}
