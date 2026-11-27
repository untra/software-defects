use serde_json::{json, Value};
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result, Worktree};

const SERVER_ID: &str = "state-lens";
const SERVER_BINARY: &str = "state-lens";
const SERVE_ARGS: [&str; 1] = ["serve"];

struct StateLens;

impl StateLens {
    fn settings(worktree: &Worktree) -> Option<LspSettings> {
        LspSettings::for_worktree(SERVER_ID, worktree).ok()
    }
}

impl zed::Extension for StateLens {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let binary = Self::settings(worktree).and_then(|s| s.binary);
        let command = binary
            .as_ref()
            .and_then(|b| b.path.clone())
            .or_else(|| worktree.which(SERVER_BINARY))
            .ok_or_else(|| {
                format!(
                    "{SERVER_BINARY} not found on PATH; install it (make install-server) or set lsp.{SERVER_ID}.binary.path"
                )
            })?;
        let args = binary
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_else(|| SERVE_ARGS.map(String::from).to_vec());
        Ok(zed::Command {
            command,
            args,
            env: worktree.shell_env(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        let mut options = json!({ "diagnostics": true });
        if let Some(Value::Object(user)) =
            Self::settings(worktree).and_then(|s| s.initialization_options)
        {
            options.as_object_mut().unwrap().extend(user);
        }
        Ok(Some(options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(Self::settings(worktree).and_then(|s| s.settings))
    }
}

zed::register_extension!(StateLens);
