//! Interactive agent launch specs for `termvox shell`.

use std::path::{Path, PathBuf};

use termvox_core::AgentInvocationOptions;

use crate::SupportedAgent;

/// Command line used to spawn an upstream agent TUI inside a PTY.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractiveLaunch {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl SupportedAgent {
    #[must_use]
    pub fn interactive_launch(
        self,
        executable: PathBuf,
        cwd: &Path,
        invocation: &AgentInvocationOptions,
        trailing_args: &[String],
    ) -> InteractiveLaunch {
        let mut args = invocation.extra_args.clone();
        if self == Self::Cursor && invocation.trust_workspace {
            args.push("-f".into());
        }
        args.extend(trailing_args.iter().cloned());
        InteractiveLaunch {
            executable,
            args,
            cwd: cwd.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CliAgent;

    #[test]
    fn cursor_interactive_includes_trust_flag_when_configured() {
        let launch = SupportedAgent::Cursor.interactive_launch(
            PathBuf::from("agent"),
            Path::new("/tmp/project"),
            &AgentInvocationOptions {
                extra_args: vec!["--model".into(), "gpt-5".into()],
                trust_workspace: true,
            },
            &[],
        );
        assert_eq!(launch.executable, PathBuf::from("agent"));
        assert_eq!(launch.cwd, PathBuf::from("/tmp/project"));
        assert!(launch.args.contains(&"-f".into()));
        assert!(launch.args.contains(&"--model".into()));
    }

    #[test]
    fn codex_interactive_has_no_exec_prompt_args() {
        let launch = CliAgent::codex().interactive_launch(
            Path::new("/repo"),
            &[],
            &AgentInvocationOptions::default(),
        );
        assert_eq!(launch.executable, PathBuf::from("codex"));
        assert!(!launch.args.iter().any(|arg| arg == "exec"));
        assert!(!launch.args.iter().any(|arg| arg.starts_with('-')));
    }

    #[test]
    fn all_agents_build_interactive_launch() {
        for agent in SupportedAgent::ALL {
            let cli = CliAgent::new(agent);
            let launch = cli.interactive_launch(
                Path::new("."),
                &[],
                &AgentInvocationOptions::default(),
            );
            assert!(!launch.executable.as_os_str().is_empty());
            assert_eq!(launch.cwd, PathBuf::from("."));
        }
    }
}
