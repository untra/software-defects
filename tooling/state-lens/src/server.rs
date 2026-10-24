use crate::{
    analysis::{language_name, Engine},
    model::{byte_offset, position, Evidence, Finding, Report, Span},
};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use url::Url;

const INDEX_IDLE_MS: u64 = 2;
const MAX_MESSAGE: usize = 16 * 1024 * 1024;
const SOURCE: &str = "state-lens";
const SITE_ORIGIN: &str = "https://badsoftware.untra.io";
const SITE_LOCALE: &str = "en";
const GUIDE_LINK_LABEL: &str = "Read the defect guide ↗";
const SEVERITY_HINT: u8 = 4;
const INLAY_MARKER: &str = "◆";
const SUPERSEDED: &str = "analysis superseded by document changes";
const CONTENT_MODIFIED: i64 = -32801;
const INTERNAL_ERROR: i64 = -32603;
const DOCUMENT_CHANGES: [&str; 4] = [
    "textDocument/didOpen",
    "textDocument/didChange",
    "textDocument/didClose",
    "workspace/didChangeWatchedFiles",
];
pub fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                length = Some(value.trim().parse::<usize>()?);
            }
        }
    }
    let length = length.context("missing Content-Length")?;
    if length > MAX_MESSAGE {
        bail!("message too large");
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}
fn send(value: Value) -> Result<()> {
    let bytes = serde_json::to_vec(&value)?;
    let mut out = io::stdout().lock();
    write!(out, "Content-Length: {}\r\n\r\n", bytes.len())?;
    out.write_all(&bytes)?;
    out.flush()?;
    Ok(())
}
fn path(uri: &str) -> Option<PathBuf> {
    Url::parse(uri).ok()?.to_file_path().ok()
}
fn lsp_range(source: &str, range: Span) -> Value {
    json!({"start":position(source,range.start),"end":position(source,range.end)})
}
fn evidence_location(engine: &Engine, evidence: &Evidence, label: &str) -> Option<Value> {
    let source = engine.source(&evidence.uri)?;
    source.get(evidence.range.start..evidence.range.end)?;
    Some(json!({
        "location":{"uri":evidence.uri,"range":lsp_range(source,evidence.range)},
        "message":format!("{label}: {}", evidence.rule_id)
    }))
}
fn defect_guide_url(family: &str, defect: &str) -> String {
    let mut url = Url::parse(SITE_ORIGIN).expect("valid site origin");
    url.path_segments_mut()
        .expect("site origin supports paths")
        .extend([SITE_LOCALE, family, defect, ""]);
    url.into()
}
fn markdown_text(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if character.is_ascii_punctuation() {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}
fn hover_description(finding: &Finding, markdown: bool) -> String {
    let description = format!(
        "{}: {} ({}, {}).\n\nSusceptibility is not a bug diagnosis.",
        finding.defect, finding.explanation, finding.origin, finding.resolution
    );
    let url = defect_guide_url(&finding.family, &finding.defect);
    if markdown {
        format!(
            "{}\n\n[{GUIDE_LINK_LABEL}](<{url}>)",
            markdown_text(&description)
        )
    } else {
        format!("{description}\n\n{GUIDE_LINK_LABEL}: {url}")
    }
}
fn diagnostics(engine: &Engine, report: &Report, uri: &str) -> Option<Value> {
    let document = report.documents.iter().find(|d| d.uri == uri)?;
    let source = engine.source(uri)?;
    let items: Vec<_> = document
        .findings
        .iter()
        .map(|f| {
            let related: Vec<_> = f
                .call_path
                .iter()
                .filter_map(|e| evidence_location(engine, e, "call path"))
                .chain(
                    f.evidence
                        .iter()
                        .filter_map(|e| evidence_location(engine, e, "evidence")),
                )
                .collect();
            json!({
                "range":lsp_range(source,f.trigger),
                "severity":SEVERITY_HINT,
                "source":SOURCE,
                "code":f.rule_id,
                "codeDescription":{"href":defect_guide_url(&f.family,&f.defect)},
                "message":format!("{}: {} ({}, {}). Susceptibility, not a bug diagnosis.",f.defect,f.explanation,f.origin,f.resolution),
                "relatedInformation":related,
                "data":{"family":f.family,"origin":f.origin,"effect":f.effect}
            })
        })
        .collect();
    Some(json!({"uri":uri,"version":document.version,"diagnostics":items}))
}
fn inlay_hints(report: &Report, source: &str, uri: &str) -> Value {
    let Some(document) = report.documents.iter().find(|d| d.uri == uri) else {
        return json!([]);
    };
    let hints: Vec<_> = document
        .functions
        .iter()
        .filter_map(|function| {
            let owned: Vec<_> = document
                .findings
                .iter()
                .filter(|f| f.function == Some(function.range))
                .collect();
            if owned.is_empty() {
                return None;
            }
            let mut families: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
            for finding in &owned {
                let count = families.entry(finding.family.as_str()).or_default();
                count.0 += 1;
                count.1 += usize::from(finding.origin == "propagated");
            }
            let label = families
                .iter()
                .map(|(family, (total, propagated))| match propagated {
                    0 => format!("{family}×{total}"),
                    _ => format!("{family}×{total} ({propagated} propagated)"),
                })
                .collect::<Vec<_>>()
                .join(" · ");
            Some(json!({
                "position":position(source,function.body.start),
                "label":format!("{INLAY_MARKER} {label}"),
                "paddingLeft":true,
                "paddingRight":true,
                "tooltip":format!("{}: state susceptibility, not a bug diagnosis", function.name)
            }))
        })
        .collect();
    json!(hints)
}
fn diagnostics_setting(settings: &Value) -> Option<bool> {
    settings["stateLens"]["diagnostics"]
        .as_bool()
        .or_else(|| settings["diagnostics"].as_bool())
}

pub fn serve() -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    let incoming = Arc::new(AtomicU64::new(0));
    let generation = incoming.clone();
    thread::spawn(move || {
        let mut input = io::stdin().lock();
        loop {
            match read_message(&mut input) {
                Ok(Some(message)) => {
                    let method = message["method"].as_str().unwrap_or("");
                    if DOCUMENT_CHANGES.contains(&method) {
                        generation.fetch_add(1, Ordering::SeqCst);
                    }
                    if sender.send(Ok(message)).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    let _ = sender.send(Err(e));
                    break;
                }
            }
        }
    });
    let mut engine = Engine::new()?;
    let mut open: BTreeMap<String, i64> = BTreeMap::new();
    let mut roots = vec![];
    let mut shutdown = false;
    let mut cancelled = BTreeSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    let mut indexing = false;
    let mut indexed_paths = BTreeSet::new();
    let mut processed = 0;
    let mut publish_diagnostics = false;
    let mut markdown_hover = false;
    let mut dirty = false;
    let mut published: BTreeMap<String, Value> = BTreeMap::new();
    loop {
        let message = match receiver.recv_timeout(Duration::from_millis(INDEX_IDLE_MS)) {
            Ok(message) => message?,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if dirty && publish_diagnostics {
                    let before = incoming.load(Ordering::SeqCst);
                    let report = engine.report();
                    if before == processed && incoming.load(Ordering::SeqCst) == before {
                        dirty = false;
                        for uri in open.keys() {
                            let Some(params) = diagnostics(&engine, &report, uri) else {
                                continue;
                            };
                            if published.get(uri) != Some(&params) {
                                send(
                                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":params}),
                                )?;
                                published.insert(uri.clone(), params);
                            }
                        }
                    }
                    continue;
                }
                if let Some(file) = queue.pop_front() {
                    let uri = Url::from_file_path(&file).unwrap().to_string();
                    if !open.contains_key(&uri) {
                        if let Err(error) = engine.load_file(&file) {
                            engine.skipped.push(format!("{}: {error}", file.display()));
                        }
                    }
                } else if indexing {
                    indexing = false;
                    dirty = true;
                    send(
                        json!({"jsonrpc":"2.0","method":"stateLens/indexing","params":{"remaining":0}}),
                    )?;
                }
                continue;
            }
        };
        let method = message["method"].as_str().unwrap_or("");
        let p = &message["params"];
        let id = message.get("id").cloned();
        if DOCUMENT_CHANGES.contains(&method) {
            processed += 1;
            dirty = true;
        }
        let result: Result<Value> = (|| match method {
            "initialize" => {
                markdown_hover = p["capabilities"]["textDocument"]["hover"]["contentFormat"]
                    .as_array()
                    .is_some_and(|formats| formats.iter().any(|format| format == "markdown"));
                if let Some(folders) = p["workspaceFolders"].as_array() {
                    for folder in folders {
                        if let Some(root) = folder["uri"].as_str().and_then(path) {
                            roots.push(root);
                        }
                    }
                }
                if roots.is_empty() {
                    if let Some(root) = p["rootUri"].as_str().and_then(path) {
                        roots.push(root);
                    }
                }
                publish_diagnostics =
                    diagnostics_setting(&p["initializationOptions"]).unwrap_or(false);
                Ok(
                    json!({"capabilities":{"positionEncoding":"utf-16","textDocumentSync":{"openClose":true,"change":2,"save":{"includeText":false}},"hoverProvider":true,"inlayHintProvider":true},"serverInfo":{"name":SOURCE,"version":env!("CARGO_PKG_VERSION")}}),
                )
            }
            "initialized" => {
                for root in &roots {
                    queue.extend(Engine::workspace_files(root)?);
                }
                indexed_paths.extend(queue.iter().cloned());
                indexing = !queue.is_empty();
                Ok(Value::Null)
            }
            "shutdown" => {
                shutdown = true;
                Ok(Value::Null)
            }
            "exit" => Ok(Value::Null),
            "$/cancelRequest" => {
                cancelled.insert(p["id"].to_string());
                Ok(Value::Null)
            }
            "textDocument/didOpen" => {
                let doc = &p["textDocument"];
                let uri = doc["uri"].as_str().context("document URI")?;
                let version = doc["version"].as_i64().context("document version")?;
                engine.set(
                    uri.into(),
                    doc["languageId"].as_str().context("language")?,
                    doc["text"].as_str().context("text")?.into(),
                    Some(version),
                )?;
                open.insert(uri.into(), version);
                Ok(Value::Null)
            }
            "textDocument/didChange" => {
                let uri = p["textDocument"]["uri"].as_str().context("document URI")?;
                let version = p["textDocument"]["version"].as_i64().context("version")?;
                if open.get(uri).is_some_and(|v| *v >= version) {
                    return Ok(Value::Null);
                }
                let mut source = engine.source(uri).context("document not open")?.to_string();
                for change in p["contentChanges"].as_array().context("changes")? {
                    let text = change["text"].as_str().context("text")?;
                    if change.get("range").is_some() {
                        let start = byte_offset(
                            &source,
                            serde_json::from_value(change["range"]["start"].clone())?,
                        )
                        .context("invalid start position")?;
                        let end = byte_offset(
                            &source,
                            serde_json::from_value(change["range"]["end"].clone())?,
                        )
                        .context("invalid end position")?;
                        if start > end {
                            bail!("reversed edit");
                        }
                        source.replace_range(start..end, text);
                    } else {
                        source = text.into();
                    }
                }
                let file = path(uri).context("file URI")?;
                let language = language_name(&file).context("language")?;
                engine.set(uri.into(), language, source, Some(version))?;
                open.insert(uri.into(), version);
                Ok(Value::Null)
            }
            "textDocument/didClose" => {
                let uri = p["textDocument"]["uri"].as_str().context("URI")?;
                open.remove(uri);
                engine.remove(uri);
                if published.remove(uri).is_some() {
                    send(
                        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}),
                    )?;
                }
                if let Some(file) = path(uri).filter(|p| p.is_file() && indexed_paths.contains(p)) {
                    engine.load_file(&file)?;
                }
                Ok(Value::Null)
            }
            "workspace/didChangeWatchedFiles" => {
                if let Some(changes) = p["changes"].as_array() {
                    for change in changes {
                        let Some(uri) = change["uri"].as_str() else {
                            continue;
                        };
                        if open.contains_key(uri) {
                            continue;
                        }
                        engine.remove(uri);
                        if let Some(file) = path(uri) {
                            if change["type"] == 3 {
                                indexed_paths.remove(&file);
                                queue.retain(|queued| queued != &file);
                            } else {
                                if change["type"] == 1 {
                                    for root in &roots {
                                        if file.starts_with(root) {
                                            indexed_paths.extend(Engine::workspace_files(root)?);
                                        }
                                    }
                                }
                                if file.is_file() && indexed_paths.contains(&file) {
                                    engine.load_file(&file)?;
                                }
                            }
                        }
                    }
                }
                Ok(Value::Null)
            }
            "stateLens/ranges" | "textDocument/hover" | "textDocument/inlayHint" => {
                let before = incoming.load(Ordering::SeqCst);
                let report = engine.report();
                if incoming.load(Ordering::SeqCst) != before || before != processed {
                    bail!(SUPERSEDED);
                }
                let uri = p["textDocument"]["uri"].as_str().context("URI")?;
                if method == "textDocument/inlayHint" {
                    let source = engine.source(uri).context("source unavailable")?;
                    return Ok(inlay_hints(&report, source, uri));
                }
                let document = report
                    .documents
                    .iter()
                    .find(|d| d.uri == uri)
                    .context("document unavailable")?;
                let source = engine.source(uri).context("source unavailable")?;
                if method == "textDocument/hover" {
                    let offset =
                        byte_offset(source, serde_json::from_value(p["position"].clone())?)
                            .context("position")?;
                    let matches: Vec<_> = document
                        .findings
                        .iter()
                        .filter(|f| f.trigger.start <= offset && offset < f.trigger.end)
                        .map(|f| hover_description(f, markdown_hover))
                        .collect();
                    if matches.is_empty() {
                        Ok(Value::Null)
                    } else {
                        Ok(
                            json!({"contents":{"kind":if markdown_hover {"markdown"} else {"plaintext"},"value":matches.join("\n\n")}}),
                        )
                    }
                } else {
                    let findings:Vec<_>=document.findings.iter().map(|f|json!({"finding":f,"triggerRange":lsp_range(source,f.trigger),"functionRange":f.function.map(|s|lsp_range(source,s))})).collect();
                    let functions:Vec<_>=document.functions.iter().map(|f|json!({"name":f.name,"range":lsp_range(source,f.range),"body":lsp_range(source,f.body)})).collect();
                    Ok(
                        json!({"schemaVersion":report.schema_version,"generation":report.generation,"uri":uri,"version":document.version,"sourceHash":document.source_hash,"findings":findings,"functions":functions,"unresolved":document.unresolved,"parseErrors":document.parse_errors,"indexing":indexing,"limitations":report.limitations}),
                    )
                }
            }
            "workspace/didChangeConfiguration" => {
                if let Some(enabled) = diagnostics_setting(&p["settings"]) {
                    publish_diagnostics = enabled;
                    dirty = enabled;
                    if !enabled {
                        for uri in std::mem::take(&mut published).into_keys() {
                            send(
                                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}),
                            )?;
                        }
                    }
                }
                Ok(Value::Null)
            }
            "textDocument/didSave" | "$/setTrace" => Ok(Value::Null),
            _ => bail!("unsupported method: {method}"),
        })();
        if let Some(id) = id {
            let response = if cancelled.remove(&id.to_string()) {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32800,"message":"Request cancelled"}})
            } else {
                match result {
                    Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
                    Err(error) => {
                        let message = error.to_string();
                        let code = if message == SUPERSEDED {
                            CONTENT_MODIFIED
                        } else {
                            INTERNAL_ERROR
                        };
                        json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
                    }
                }
            };
            send(response)?;
        } else if let Err(error) = result {
            send(
                json!({"jsonrpc":"2.0","method":"window/logMessage","params":{"type":1,"message":error.to_string()}}),
            )?;
        }
        if method == "exit" {
            if !shutdown {
                bail!("exit before shutdown");
            }
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_urls_follow_taxonomy_routes() {
        for (family, defect) in [
            ("mutated-state", "var-let-mut"),
            ("mutated-state", "impure-functions"),
            ("unstructured-state", "type-errors"),
            ("shared-state", "threading-mutexes"),
        ] {
            assert_eq!(
                defect_guide_url(family, defect),
                format!("https://badsoftware.untra.io/en/{family}/{defect}/")
            );
        }
    }

    #[test]
    fn hover_markdown_escapes_explanation_without_escaping_the_link() {
        let mut engine = Engine::new().unwrap();
        engine
            .set(
                "file:///clock.ts".into(),
                "typescript",
                "function clock(){return Date.now();}".into(),
                Some(1),
            )
            .unwrap();
        let mut finding = engine.analyze().documents.remove(0).findings.remove(0);
        finding.explanation = "[Open](command:unexpected) <b>text</b> *emphasis*".into();
        let hover = hover_description(&finding, true);
        assert!(hover.contains(r"\[Open\]\(command\:unexpected\) \<b\>text\<\/b\> \*emphasis\*"));
        assert!(hover.contains("[Read the defect guide ↗](<https://badsoftware.untra.io/en/mutated-state/time-money-entropy/>)"));
        assert!(hover_description(&finding, false).contains(&finding.explanation));
    }
}
