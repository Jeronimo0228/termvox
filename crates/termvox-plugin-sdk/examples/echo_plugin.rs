use async_trait::async_trait;
use serde_json::{Value, json};
use termvox_plugin_sdk::{
    InitializeParams, PluginCapabilities, PluginHandler, PluginManifest, Result, SendParams,
    StartResult, serve,
};

struct EchoPlugin;

#[async_trait]
impl PluginHandler for EchoPlugin {
    async fn initialize(&mut self, params: InitializeParams) -> Result<PluginManifest> {
        Ok(PluginManifest {
            id: "dev.termvox.echo".into(),
            name: "TermVox Echo Example".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: params.protocol_version,
            capabilities: PluginCapabilities {
                streaming: false,
                resume: false,
                cancellation: true,
            },
        })
    }

    async fn probe(&mut self) -> Result<Value> {
        Ok(json!({"ready": true}))
    }

    async fn start(&mut self) -> Result<StartResult> {
        Ok(StartResult {
            session_id: "echo-session".into(),
        })
    }

    async fn send(&mut self, params: SendParams) -> Result<Value> {
        Ok(json!({"text": params.prompt}))
    }

    async fn cancel(&mut self, _session_id: String) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    serve(
        EchoPlugin,
        tokio::io::stdin(),
        tokio::io::stdout(),
        1024 * 1024,
    )
    .await
}
