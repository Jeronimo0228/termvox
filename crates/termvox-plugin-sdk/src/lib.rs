//! Versioned, out-of-process plugin protocol for `TermVox`.
#![allow(clippy::missing_errors_doc)]

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use termvox_core::{
    AgentAdapter, AgentCapabilities, AgentEvent, AgentEventStream, AgentInfo, AgentRequest,
    AgentSession, PluginConfig, TermVoxError,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;

pub const PROTOCOL_VERSION: u32 = 1;
pub const PROTOCOL_SEMVER: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: u32,
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCapabilities {
    pub streaming: bool,
    pub resume: bool,
    pub cancellation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializeParams {
    pub protocol_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartResult {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendParams {
    pub session_id: String,
    pub prompt: String,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventNotification {
    pub session_id: String,
    pub event: Value,
}

#[derive(Debug, Clone)]
pub struct PluginSpawnOptions {
    pub cwd: PathBuf,
    pub env_allowlist: Vec<String>,
    pub timeout: Duration,
    pub max_frame_bytes: usize,
}

impl Default for PluginSpawnOptions {
    fn default() -> Self {
        Self {
            cwd: std::env::temp_dir().join("termvox-plugin"),
            env_allowlist: Vec::new(),
            timeout: Duration::from_secs(30),
            max_frame_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("plugin protocol failed: {0}")]
    Protocol(String),
    #[error("plugin returned {code}: {message}")]
    Remote { code: i64, message: String },
    #[error("plugin process did not expose stdin/stdout")]
    MissingPipe,
    #[error("plugin call timed out")]
    Timeout,
    #[error("plugin frame exceeded configured limit")]
    FrameTooLarge,
}

pub type Result<T> = std::result::Result<T, PluginError>;

pub struct PluginClient {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    next_id: AtomicU64,
    manifest: PluginManifest,
    timeout: Duration,
    max_frame_bytes: usize,
    events: VecDeque<EventNotification>,
}

impl PluginClient {
    pub async fn spawn(executable: &Path, args: &[String]) -> Result<Self> {
        Self::spawn_with(executable, args, PluginSpawnOptions::default()).await
    }

    pub async fn spawn_with(
        executable: &Path,
        args: &[String],
        options: PluginSpawnOptions,
    ) -> Result<Self> {
        if !executable.is_file() {
            return Err(PluginError::Protocol(format!(
                "plugin executable does not exist: {}",
                executable.display()
            )));
        }
        let executable = std::fs::canonicalize(executable)?;
        tokio::fs::create_dir_all(&options.cwd).await?;
        let mut command = Command::new(&executable);
        command
            .args(args)
            .current_dir(&options.cwd)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        for name in &options.env_allowlist {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        let mut child = command.spawn()?;
        let stdin = child.stdin.take().ok_or(PluginError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(PluginError::MissingPipe)?;
        let placeholder = PluginManifest {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: PluginCapabilities::default(),
        };
        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            next_id: AtomicU64::new(1),
            manifest: placeholder,
            timeout: options.timeout,
            max_frame_bytes: options.max_frame_bytes,
            events: VecDeque::new(),
        };
        let value = client
            .call(
                "initialize",
                serde_json::to_value(InitializeParams {
                    protocol_version: PROTOCOL_VERSION,
                })
                .map_err(|error| PluginError::Protocol(error.to_string()))?,
            )
            .await?;
        let manifest: PluginManifest = serde_json::from_value(value)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        if manifest.protocol_version != PROTOCOL_VERSION {
            return Err(PluginError::Protocol(format!(
                "unsupported protocol version {}, expected {PROTOCOL_VERSION}",
                manifest.protocol_version
            )));
        }
        client.manifest = manifest;
        Ok(client)
    }

    #[must_use]
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        tokio::time::timeout(self.timeout, self.call_inner(method, params))
            .await
            .map_err(|_| PluginError::Timeout)?
    }

    async fn call_inner(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_owned(),
            params,
        };
        let mut line = serde_json::to_vec(&request)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        if line.len() > self.max_frame_bytes {
            return Err(PluginError::FrameTooLarge);
        }
        line.push(b'\n');
        self.stdin.write_all(&line).await?;
        self.stdin.flush().await?;
        loop {
            let line = self
                .stdout
                .next_line()
                .await?
                .ok_or_else(|| PluginError::Protocol("plugin closed stdout".into()))?;
            if line.len() > self.max_frame_bytes {
                return Err(PluginError::FrameTooLarge);
            }
            if let Ok(notification) = serde_json::from_str::<RpcNotification>(&line)
                && notification.jsonrpc == "2.0"
                && notification.method == "event"
            {
                let event = serde_json::from_value(notification.params)
                    .map_err(|error| PluginError::Protocol(error.to_string()))?;
                self.events.push_back(event);
                continue;
            }
            let response: RpcResponse = serde_json::from_str(&line)
                .map_err(|error| PluginError::Protocol(error.to_string()))?;
            if response.id != id {
                continue;
            }
            if let Some(error) = response.error {
                return Err(PluginError::Remote {
                    code: error.code,
                    message: error.message,
                });
            }
            return response
                .result
                .ok_or_else(|| PluginError::Protocol("response has no result".into()));
        }
    }

    pub async fn probe(&mut self) -> Result<Value> {
        self.call("probe", Value::Null).await
    }

    pub async fn start(&mut self) -> Result<StartResult> {
        let value = self.call("start", Value::Null).await?;
        serde_json::from_value(value).map_err(|error| PluginError::Protocol(error.to_string()))
    }

    pub async fn send(&mut self, params: &SendParams) -> Result<Value> {
        let value = serde_json::to_value(params)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        self.call("send", value).await
    }

    pub async fn cancel(&mut self, session_id: &str) -> Result<()> {
        self.call("cancel", serde_json::json!({ "session_id": session_id }))
            .await?;
        Ok(())
    }

    pub fn next_buffered_event(&mut self) -> Option<EventNotification> {
        self.events.pop_front()
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let _ = self.call("shutdown", Value::Null).await;
        self.child.kill().await?;
        Ok(())
    }
}

#[async_trait]
pub trait PluginHandler: Send {
    async fn initialize(&mut self, params: InitializeParams) -> Result<PluginManifest>;
    async fn probe(&mut self) -> Result<Value>;
    async fn start(&mut self) -> Result<StartResult>;
    async fn send(&mut self, params: SendParams) -> Result<Value>;
    async fn cancel(&mut self, session_id: String) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}

pub async fn serve<H, R, W>(
    mut handler: H,
    reader: R,
    mut writer: W,
    max_frame_bytes: usize,
) -> Result<()>
where
    H: PluginHandler,
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        if line.len() > max_frame_bytes {
            return Err(PluginError::FrameTooLarge);
        }
        let request: Value = serde_json::from_str(&line)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        let id = request
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| PluginError::Protocol("request has no numeric id".into()))?;
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| PluginError::Protocol("request has no method".into()))?;
        let params = request.get("params").cloned().unwrap_or(Value::Null);
        let result = match method {
            "initialize" => {
                let params = serde_json::from_value(params)
                    .map_err(|error| PluginError::Protocol(error.to_string()))?;
                serde_json::to_value(handler.initialize(params).await?)
            }
            "probe" => Ok(handler.probe().await?),
            "start" => serde_json::to_value(handler.start().await?),
            "send" => {
                let params = serde_json::from_value(params)
                    .map_err(|error| PluginError::Protocol(error.to_string()))?;
                Ok(handler.send(params).await?)
            }
            "cancel" => {
                let session_id = params
                    .get("session_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| PluginError::Protocol("cancel requires session_id".into()))?;
                handler.cancel(session_id.to_owned()).await?;
                Ok(Value::Null)
            }
            "shutdown" => {
                handler.shutdown().await?;
                Ok(Value::Null)
            }
            _ => Err(serde_json::Error::io(std::io::Error::other(format!(
                "unknown method: {method}"
            )))),
        }
        .map_err(|error| PluginError::Protocol(error.to_string()))?;
        let mut frame = serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }))
        .map_err(|error| PluginError::Protocol(error.to_string()))?;
        if frame.len() > max_frame_bytes {
            return Err(PluginError::FrameTooLarge);
        }
        frame.push(b'\n');
        writer.write_all(&frame).await?;
        writer.flush().await?;
        if method == "shutdown" {
            break;
        }
    }
    Ok(())
}

pub struct PluginAgentAdapter {
    config: PluginConfig,
    client: Mutex<Option<PluginClient>>,
}

impl PluginAgentAdapter {
    #[must_use]
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            client: Mutex::new(None),
        }
    }

    async fn connect(&self) -> Result<PluginClient> {
        PluginClient::spawn_with(
            &self.config.executable,
            &self.config.args,
            PluginSpawnOptions {
                cwd: std::env::temp_dir()
                    .join("termvox-plugins")
                    .join(&self.config.id),
                env_allowlist: self.config.env_allowlist.clone(),
                timeout: Duration::from_secs(self.config.timeout_seconds),
                max_frame_bytes: self.config.max_frame_bytes,
            },
        )
        .await
    }
}

#[async_trait]
impl AgentAdapter for PluginAgentAdapter {
    fn id(&self) -> &'static str {
        "plugin"
    }

    async fn probe(&self) -> AgentInfo {
        match self.connect().await {
            Ok(mut client) => {
                let manifest = client.manifest().clone();
                let ready = client.probe().await.is_ok();
                let _ = client.shutdown().await;
                AgentInfo {
                    id: self.config.id.clone(),
                    executable: self.config.executable.display().to_string(),
                    installed: ready,
                    version: Some(manifest.version),
                    capabilities: AgentCapabilities {
                        structured_output: true,
                        streaming: manifest.capabilities.streaming,
                        resume: manifest.capabilities.resume,
                    },
                }
            }
            Err(_) => AgentInfo {
                id: self.config.id.clone(),
                executable: self.config.executable.display().to_string(),
                installed: false,
                version: None,
                capabilities: AgentCapabilities::default(),
            },
        }
    }

    async fn start(&self) -> termvox_core::Result<AgentSession> {
        let mut client = self
            .connect()
            .await
            .map_err(|error| TermVoxError::Agent(error.to_string()))?;
        let remote = client
            .start()
            .await
            .map_err(|error| TermVoxError::Agent(error.to_string()))?;
        let session = AgentSession::default();
        session.set_remote_id(remote.session_id).await;
        *self.client.lock().await = Some(client);
        Ok(session)
    }

    async fn send_prompt(
        &self,
        session: &AgentSession,
        request: AgentRequest,
        cancel: CancellationToken,
    ) -> termvox_core::Result<AgentEventStream> {
        if cancel.is_cancelled() {
            return Err(TermVoxError::Cancelled);
        }
        let session_id = session
            .remote_id()
            .await
            .ok_or_else(|| TermVoxError::Agent("plugin session was not started".into()))?;
        let mut guard = self.client.lock().await;
        let client = guard
            .as_mut()
            .ok_or_else(|| TermVoxError::Agent("plugin process is not connected".into()))?;
        let params = SendParams {
            session_id: session_id.clone(),
            prompt: request.prompt,
            cwd: request.cwd,
        };
        let result = tokio::select! {
            () = cancel.cancelled() => {
                let _ = client.cancel(&session_id).await;
                return Err(TermVoxError::Cancelled);
            }
            result = client.send(&params) => result
                .map_err(|error| TermVoxError::Agent(error.to_string()))?,
        };
        let (tx, rx) = mpsc::channel(64);
        while let Some(notification) = client.next_buffered_event() {
            if let Ok(event) = serde_json::from_value::<AgentEvent>(notification.event) {
                let _ = tx.send(Ok(event)).await;
            }
        }
        let event = serde_json::from_value::<AgentEvent>(result.clone()).unwrap_or_else(|_| {
            AgentEvent::Completed {
                result: result
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            }
        });
        let _ = tx.send(Ok(event)).await;
        drop(tx);
        Ok(rx)
    }

    async fn cancel(&self, session: &AgentSession) -> termvox_core::Result<()> {
        if let (Some(client), Some(session_id)) =
            (self.client.lock().await.as_mut(), session.remote_id().await)
        {
            client
                .cancel(&session_id)
                .await
                .map_err(|error| TermVoxError::Agent(error.to_string()))?;
        }
        Ok(())
    }

    async fn shutdown(&self) -> termvox_core::Result<()> {
        if let Some(client) = self.client.lock().await.take() {
            client
                .shutdown()
                .await
                .map_err(|error| TermVoxError::Agent(error.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips() {
        let manifest = PluginManifest {
            id: "example".into(),
            name: "Example".into(),
            version: "1.0.0".into(),
            protocol_version: PROTOCOL_VERSION,
            capabilities: PluginCapabilities {
                streaming: true,
                ..PluginCapabilities::default()
            },
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert_eq!(
            serde_json::from_str::<PluginManifest>(&json).unwrap(),
            manifest
        );
    }

    struct TestHandler;

    #[async_trait]
    impl PluginHandler for TestHandler {
        async fn initialize(&mut self, _params: InitializeParams) -> Result<PluginManifest> {
            Ok(PluginManifest {
                id: "test".into(),
                name: "Test".into(),
                version: "1.0.0".into(),
                protocol_version: PROTOCOL_VERSION,
                capabilities: PluginCapabilities::default(),
            })
        }

        async fn probe(&mut self) -> Result<Value> {
            Ok(serde_json::json!({"ready": true}))
        }

        async fn start(&mut self) -> Result<StartResult> {
            Ok(StartResult {
                session_id: "session".into(),
            })
        }

        async fn send(&mut self, params: SendParams) -> Result<Value> {
            Ok(serde_json::json!({"text": params.prompt}))
        }

        async fn cancel(&mut self, _session_id: String) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn server_conformance_initialize_and_shutdown() {
        let (mut client, server) = tokio::io::duplex(4_096);
        let (server_read, server_write) = tokio::io::split(server);
        let task = tokio::spawn(serve(TestHandler, server_read, server_write, 1_024));
        client
            .write_all(
                b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocol_version\":1}}\n",
            )
            .await
            .unwrap();
        let mut lines = BufReader::new(&mut client).lines();
        let response = lines.next_line().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&response).unwrap()["result"]["id"],
            "test"
        );
        drop(lines);
        client
            .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"shutdown\",\"params\":null}\n")
            .await
            .unwrap();
        task.await.unwrap().unwrap();
    }
}
