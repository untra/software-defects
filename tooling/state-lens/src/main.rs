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
        Some("scan-isolated") => {
            let paths: Vec<String> = serde_json::from_reader(std::io::stdin())?;
            let mut engine = Engine::new()?;
            for path in paths {
                let path = Path::new(&path).canonicalize()?;
                let language = state_lens::analysis::language_name(&path)
                    .ok_or_else(|| anyhow::anyhow!("unsupported language: {}", path.display()))?;
                let uri = url::Url::from_file_path(&path)
                    .map_err(|_| anyhow::anyhow!("invalid path"))?
                    .to_string();
                engine.set(
                    uri.clone(),
                    language,
                    std::fs::read_to_string(&path)?,
                    Some(1),
                )?;
                println!("{}", serde_json::to_string(&engine.analyze())?);
                engine.remove(&uri);
            }
            Ok(())
        }
        Some("scan") => {
            let mut engine = Engine::new()?;
            engine.load(Path::new(args.get(2).map(String::as_str).unwrap_or(".")))?;
            println!("{}", serde_json::to_string_pretty(&engine.analyze())?);
            Ok(())
        }
        Some("repair-check") => {
            let path = Path::new(
                args.get(2)
                    .ok_or_else(|| anyhow::anyhow!("expected source path"))?,
            );
            let language = state_lens::analysis::language_name(path)
                .ok_or_else(|| anyhow::anyhow!("unsupported language"))?;
            let source = std::fs::read_to_string(path)?;
            let option = |name: &str| {
                args.iter()
                    .position(|a| a == name)
                    .and_then(|i| args.get(i + 1))
                    .cloned()
            };
            let finding = match option("--finding") {
                Some(index) => index.parse()?,
                None => state_lens::repair::first_finding(&source, language)
                    .ok_or_else(|| anyhow::anyhow!("no object-identity finding"))?,
            };
            let result = state_lens::repair::check(
                &source,
                language,
                &path.display().to_string(),
                finding,
                option("--hash").as_deref(),
            );
            println!("{}", serde_json::to_string_pretty(&result)?);
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
                "Usage: state-lens scan [PATH] | scan-isolated < PATHS.json | semantic-facts PATH | repair-check PATH [--finding N] [--hash SHA256] | check-queries | score EXPECTATIONS.json | serve"
            );
            Ok(())
        }
    }
}
