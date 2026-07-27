//! Safe, streaming subprocess adapters for supported coding-agent CLIs.
#![allow(clippy::too_many_lines)]

use std::{path::PathBuf, process::Stdio, time::Duration};

use async_trait::async_trait;
use termvox_core::{
    AgentAdapter, AgentCapabilities, AgentEvent, AgentEventStream, AgentInfo, AgentRequest,
    AgentSession, PermissionProfile, Result, TermVoxError,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

mod parsers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedAgent {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Aider,
    Amp,
}

impl SupportedAgent {
    pub const ALL: [Self; 6] = [
        Self::Codex,
        Self::Claude,
        Self::Cursor,
        Self::Gemini,
        Self::Aider,
        Self::Amp,
    ];

    const fn id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
            Self::Aider => "aider",
            Self::Amp => "amp",
        }
    }

    const fn executable(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Cursor => "agent",
            Self::Gemini => "gemini",
            Self::Aider => "aider",
            Self::Amp => "amp",
        }
    }

    const fn capabilities(self) -> AgentCapabilities {
        AgentCapabilities {
            structured_output: !matches!(self, Self::Aider),
            streaming: !matches!(self, Self::Aider),
            resume: !matches!(self, Self::Aider),
        }
    }
}

#[derive(Clone)]
pub struct CliAgent {
    kind: SupportedAgent,
    executable: PathBuf,
}

impl CliAgent {
    #[must_use]
    pub fn new(kind: SupportedAgent) -> Self {
        Self {
            kind,
            executable: PathBuf::from(kind.executable()),
        }
    }

    #[must_use]
    pub fn codex() -> Self {
        Self::new(SupportedAgent::Codex)
    }

    #[must_use]
    pub fn claude() -> Self {
        Self::new(SupportedAgent::Claude)
    }

    #[must_use]
    pub fn cursor() -> Self {
        Self::new(SupportedAgent::Cursor)
    }

    #[must_use]
    pub fn gemini() -> Self {
        Self::new(SupportedAgent::Gemini)
    }

    #[must_use]
    pub fn aider() -> Self {
        Self::new(SupportedAgent::Aider)
    }

    #[must_use]
    pub fn amp() -> Self {
        Self::new(SupportedAgent::Amp)
    }

    #[must_use]
    pub fn with_executable(kind: SupportedAgent, executable: PathBuf) -> Self {
        Self { kind, executable }
    }

    async fn command(&self, session: &AgentSession, request: &AgentRequest) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&request.cwd)
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let remote_id = session.remote_id().await;
        match self.kind {
            SupportedAgent::Codex => {
                command.arg("exec");
                if let Some(id) = remote_id {
                    command.args(["resume", &id]);
                }
                command.args(["--json", &request.prompt]);
            }
            SupportedAgent::Claude => {
                command.args([
                    "-p",
                    &request.prompt,
                    "--output-format",
                    "stream-json",
                    "--verbose",
                ]);
                if let Some(id) = remote_id {
                    command.args(["--resume", &id]);
                }
            }
            SupportedAgent::Cursor | SupportedAgent::Gemini => {
                command.args(["-p", &request.prompt, "--output-format", "stream-json"]);
                if let Some(id) = remote_id {
                    command.args(["--resume", &id]);
                }
            }
            SupportedAgent::Aider => {
                command.args(["--message", &request.prompt]);
            }
            SupportedAgent::Amp => {
                command.args(["-x", &request.prompt, "--stream-json"]);
                if let Some(id) = remote_id {
                    command.args(["--resume", &id]);
                }
            }
        }
        // Permission elevation is deliberately not translated to undocumented flags.
        if request.permission_profile != PermissionProfile::Safe {
            tracing::warn!(
                agent = self.kind.id(),
                profile = ?request.permission_profile,
                "agent permission profile requested; only documented defaults are passed"
            );
        }
        command
    }
}

#[async_trait]
impl AgentAdapter for CliAgent {
    fn id(&self) -> &'static str {
        self.kind.id()
    }

    async fn probe(&self) -> AgentInfo {
        let path = if self.executable.components().count() > 1 && self.executable.is_file() {
            Some(self.executable.clone())
        } else {
            which::which(&self.executable).ok()
        };
        let version = if path.is_some() {
            tokio::time::timeout(
                Duration::from_secs(3),
                Command::new(&self.executable).arg("--version").output(),
            )
            .await
            .ok()
            .and_then(std::result::Result::ok)
            .filter(|output| output.status.success())
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                if stdout.is_empty() {
                    String::from_utf8_lossy(&output.stderr).trim().to_owned()
                } else {
                    stdout
                }
            })
        } else {
            None
        };
        AgentInfo {
            id: self.id().to_owned(),
            executable: path
                .as_deref()
                .unwrap_or(&self.executable)
                .display()
                .to_string(),
            installed: path.is_some(),
            version,
            capabilities: if path.is_some() {
                self.kind.capabilities()
            } else {
                AgentCapabilities::default()
            },
        }
    }

    async fn start(&self) -> Result<AgentSession> {
        Ok(AgentSession::default())
    }

    async fn send_prompt(
        &self,
        session: &AgentSession,
        request: AgentRequest,
        cancel: CancellationToken,
    ) -> Result<AgentEventStream> {
        if !request.cwd.is_dir() {
            return Err(TermVoxError::Agent(format!(
                "working directory does not exist: {}",
                request.cwd.display()
            )));
        }
        let mut child = self.command(session, &request).await.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TermVoxError::Agent("agent stdout was not piped".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TermVoxError::Agent("agent stderr was not piped".into()))?;
        let kind = self.kind;
        let session = session.clone();
        let limits = request.limits;
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let stderr_limit = limits.max_output_bytes;
            let stderr_task = tokio::spawn(async move {
                let mut bytes = Vec::new();
                let _ = stderr
                    .take((stderr_limit.saturating_add(1)) as u64)
                    .read_to_end(&mut bytes)
                    .await;
                bytes
            });
            let mut lines = BufReader::new(stdout).lines();
            let mut output_bytes = 0_usize;
            let deadline = tokio::time::sleep(limits.agent_timeout());
            tokio::pin!(deadline);
            let mut terminal_error = None;

            loop {
                tokio::select! {
                    () = cancel.cancelled() => {
                        terminal_error = Some(TermVoxError::Cancelled);
                        break;
                    }
                    () = &mut deadline => {
                        terminal_error = Some(TermVoxError::Agent("agent timed out".into()));
                        break;
                    }
                    line = lines.next_line() => match line {
                        Ok(Some(line)) => {
                            output_bytes = output_bytes.saturating_add(line.len() + 1);
                            if line.len() > limits.max_json_frame_bytes {
                                terminal_error = Some(TermVoxError::Agent("agent JSON frame exceeded configured limit".into()));
                                break;
                            }
                            if output_bytes > limits.max_output_bytes {
                                terminal_error = Some(TermVoxError::Agent("agent output exceeded configured limit".into()));
                                break;
                            }
                            if let Some(event) = parse_line(kind, &line) {
                                if let AgentEvent::Started { session_id: Some(id) } = &event {
                                    session.set_remote_id(id.clone()).await;
                                }
                                if tx.send(Ok(event)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(error) => {
                            terminal_error = Some(TermVoxError::Io(error));
                            break;
                        }
                    }
                }
            }
            if terminal_error.is_some() {
                let _ = child.kill().await;
            }
            let status = child.wait().await;
            let stderr = stderr_task.await.unwrap_or_default();
            if let Some(error) = terminal_error {
                let _ = tx.send(Err(error)).await;
                return;
            }
            match status {
                Ok(status) if status.success() => {
                    if kind == SupportedAgent::Aider && output_bytes == 0 {
                        let _ = tx
                            .send(Ok(AgentEvent::Completed {
                                result: String::new(),
                            }))
                            .await;
                    }
                }
                Ok(status) => {
                    let detail = String::from_utf8_lossy(&stderr).trim().to_owned();
                    let message = if detail.is_empty() {
                        format!("{} exited with {status}", kind.id())
                    } else if stderr.len() > limits.max_output_bytes {
                        let truncated = detail
                            .chars()
                            .take(limits.max_output_bytes)
                            .collect::<String>();
                        format!("{truncated} (stderr truncated)")
                    } else {
                        detail
                    };
                    let _ = tx.send(Ok(AgentEvent::Failed { message })).await;
                }
                Err(error) => {
                    let _ = tx.send(Err(TermVoxError::Io(error))).await;
                }
            }
        });
        Ok(rx)
    }
}

fn parse_line(kind: SupportedAgent, line: &str) -> Option<AgentEvent> {
    match kind {
        SupportedAgent::Codex => parsers::codex::parse(line),
        SupportedAgent::Claude => parsers::claude::parse(line),
        SupportedAgent::Cursor => parsers::cursor::parse(line),
        SupportedAgent::Gemini => parsers::gemini::parse(line),
        SupportedAgent::Aider => parsers::aider::parse(line),
        SupportedAgent::Amp => parsers::amp::parse(line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termvox_core::RuntimeLimits;

    #[test]
    fn all_structured_adapters_parse_forward_compatible_jsonl() {
        let fixtures = [
            (
                SupportedAgent::Codex,
                include_str!("../tests/fixtures/codex.jsonl"),
            ),
            (
                SupportedAgent::Claude,
                include_str!("../tests/fixtures/claude.jsonl"),
            ),
            (
                SupportedAgent::Cursor,
                include_str!("../tests/fixtures/cursor.jsonl"),
            ),
            (
                SupportedAgent::Gemini,
                include_str!("../tests/fixtures/gemini.jsonl"),
            ),
            (
                SupportedAgent::Amp,
                include_str!("../tests/fixtures/amp.jsonl"),
            ),
        ];
        for (kind, fixture) in fixtures {
            let event = parse_line(kind, fixture.lines().nth(1).unwrap());
            assert_eq!(
                event,
                Some(AgentEvent::Completed {
                    result: "done".into()
                })
            );
        }
    }

    #[tokio::test]
    async fn commands_never_add_auto_approval_flags() {
        let request = AgentRequest {
            prompt: "hello".into(),
            cwd: std::env::temp_dir(),
            limits: RuntimeLimits::default(),
            permission_profile: PermissionProfile::Safe,
        };
        for kind in SupportedAgent::ALL {
            let command = CliAgent::new(kind)
                .command(&AgentSession::default(), &request)
                .await;
            let args = command
                .as_std()
                .get_args()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(!args.contains("--dangerously"));
            assert!(!args.contains("--yolo"));
            assert!(!args.contains("--force"));
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stream_is_incremental_and_honors_output_limit() {
        use std::os::unix::fs::PermissionsExt;

        let executable =
            std::env::temp_dir().join(format!("termvox-agent-limit-{}.sh", std::process::id()));
        std::fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' '{\"type\":\"result\",\"result\":\"one\"}' '{\"type\":\"result\",\"result\":\"two\"}'\n",
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).unwrap();
        let agent = CliAgent::with_executable(SupportedAgent::Codex, executable.clone());
        let limits = RuntimeLimits {
            max_output_bytes: 45,
            ..RuntimeLimits::default()
        };
        let mut events = agent
            .send_prompt(
                &AgentSession::default(),
                AgentRequest {
                    prompt: "hello".into(),
                    cwd: std::env::temp_dir(),
                    limits,
                    permission_profile: PermissionProfile::Safe,
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(matches!(
            events.recv().await,
            Some(Ok(AgentEvent::Completed { result })) if result == "one"
        ));
        assert!(matches!(
            events.recv().await,
            Some(Err(TermVoxError::Agent(message))) if message.contains("limit")
        ));
        let _ = std::fs::remove_file(executable);
    }
}
