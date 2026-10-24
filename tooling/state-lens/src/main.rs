use anyhow::{bail, Result};
use state_lens::{analysis::Engine, corpus, server};
use std::path::Path;
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("serve") => server::serve(),
        Some("check-queries") => {
            Engine::new()?;
            println!("All queries compile against pinned grammars.");
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
                "Usage: state-lens scan [PATH] | check-queries | score EXPECTATIONS.json | serve"
            );
            Ok(())
        }
    }
}
