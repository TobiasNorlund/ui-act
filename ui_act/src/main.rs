use crate::agent::AnthropicAgent;

mod env;
mod agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = env::MPXEnvironment::create()?;
    let agent = AnthropicAgent::create().await?;

    tokio::select! {
        result = agent.run(env, "Calculate 1+2 in the Calculator window. Here is a screenshot of my desktop") => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, shutting down gracefully...");
            // When main returns, env will be dropped, and XInputMaster::drop will run.
        }
    }

    Ok(())
}