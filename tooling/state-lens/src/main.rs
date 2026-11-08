use anyhow::{bail, Result};
use state_lens::{analysis::Engine, corpus, server};
use std::path::Path;
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("capabilities") => {
            println!(
                "{}",
                serde_json::json!({"languages": state_lens::analysis::SUPPORTED_LANGUAGES})
            );
            Ok(())
        }
        Some("serve") => server::serve(),
        Some("check-queries") => {
            Engine::new()?;
            println!("All queries compile against pinned grammars.");
            Ok(())
        }
        Some("semantic-facts") => {
            let path = Path::new(
                args.get(2)
                    .ok_or_else(|| anyhow::anyhow!("expected source path"))?,
            );
            let language = state_lens::analysis::language_name(path)
                .ok_or_else(|| anyhow::anyhow!("unsupported language"))?;
            let mut facts =
                state_lens::semantic::analyze(language, &std::fs::read_to_string(path)?)?;
            facts["uri"] = serde_json::Value::String(
                url::Url::from_file_path(path.canonicalize()?)
                    .map_err(|_| anyhow::anyhow!("invalid source path"))?
                    .to_string(),
            );
            println!("{}", serde_json::to_string_pretty(&facts)?);
            Ok(())
        }
        Some("scan") => {
            let mut engine = Engine::new()?;
            engine.load(Path::new(args.get(2).map(String::as_str).unwrap_or(".")))?;
            println!("{}", serde_json::to_string_pretty(&engine.analyze())?);
            Ok(())
        }
        Some("score") => {
            let scores = corpus::score(Path::new(
                args.get(2)
                    .ok_or_else(|| anyhow::anyhow!("expected sidecar path"))?,
            ))?;
            println!("{}", serde_json::to_string_pretty(&scores)?);
            if scores
                .iter()
                .any(|s| s.false_positive > 0 || s.false_negative > 0)
            {
                bail!("expectations differ");
            }
            Ok(())
        }
        _ => {
            eprintln!(
                "Usage: state-lens scan [PATH] | semantic-facts PATH | check-queries | score EXPECTATIONS.json | serve"
            );
            Ok(())
        }
    }
}
