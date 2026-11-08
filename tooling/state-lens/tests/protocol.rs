use serde_json::{json, Value};
use state_lens::{model::position, server::read_message};
use std::{
    io::{BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    thread,
    time::Duration,
};
const PUBLISH: &str = "textDocument/publishDiagnostics";
const CLOCK_URI: &str = "file:///state-lens-tests/clock.ts";
const CLOCK_SOURCE: &str = "function clock() { return Date.now(); }\n";
struct Server {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next: u64,
    notifications: Vec<Value>,
}
impl Server {
    fn start() -> Self {
        Self::start_with(Value::Null)
    }
    fn start_with(options: Value) -> Self {
        Self::start_with_capabilities(options, json!({}))
    }
    fn start_with_capabilities(options: Value, capabilities: Value) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_state-lens"))
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut server = Self {
            child,
            input,
            output,
            next: 0,
            notifications: vec![],
        };
        let result = server.request(
            "initialize",
            json!({"capabilities":capabilities,"initializationOptions":options}),
        );
        assert_eq!(
            result["result"]["capabilities"]["positionEncoding"],
            "utf-16"
        );
        assert_eq!(
            result["result"]["capabilities"]["experimental"]["stateLens"]["stateAnnotations"],
            2
        );
        server.notify("initialized", json!({}));
        server
    }
    fn send(&mut self, message: Value) {
        let bytes = serde_json::to_vec(&message).unwrap();
        write!(self.input, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
        self.input.write_all(&bytes).unwrap();
        self.input.flush().unwrap();
    }
    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({"jsonrpc":"2.0","method":method,"params":params}));
    }
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let id = self.next;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        loop {
            let response = read_message(&mut self.output).unwrap().unwrap();
            if response["id"] == id {
                return response;
            }
            if response.get("id").is_none() {
                self.notifications.push(response);
            }
        }
    }
    fn published(&mut self, uri: &str) -> Value {
        loop {
            if let Some(index) = self
                .notifications
                .iter()
                .position(|n| n["method"] == PUBLISH && n["params"]["uri"] == uri)
            {
                return self.notifications.remove(index)["params"].clone();
            }
            let message = read_message(&mut self.output).unwrap().unwrap();
            self.notifications.push(message);
        }
    }
    fn open_clock(&mut self) {
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument":{"uri":CLOCK_URI,"languageId":"typescript","version":1,"text":CLOCK_SOURCE}}),
        );
    }
    fn stop(&mut self) {
        assert!(self.request("shutdown", Value::Null).get("error").is_none());
        self.notify("exit", Value::Null);
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn document_lifecycle_incremental_unicode_edit_and_hover() {
    let mut server = Server::start();
    let uri = "file:///state-lens-tests/clock.ts";
    let source = "// 😀\r\nfunction clock() { return Date.now(); }\r\n";
    server.notify(
        "textDocument/didOpen",
        json!({"textDocument":{"uri":uri,"languageId":"typescript","version":1,"text":source}}),
    );
    let before = server.request("stateLens/ranges", json!({"textDocument":{"uri":uri}}));
    assert_eq!(before["result"]["findings"].as_array().unwrap().len(), 1);
    let start = source.find("Date.now()").unwrap();
    let end = start + "Date.now()".len();
    let hover = server.request(
        "textDocument/hover",
        json!({"textDocument":{"uri":uri},"position":position(source,start)}),
    );
    assert!(hover["result"]["contents"]["value"]
        .as_str()
        .unwrap()
        .contains("Clock"));
    server.notify("textDocument/didChange",json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"range":{"start":position(source,start),"end":position(source,end)},"text":"1"}]}));
    let after = server.request("stateLens/ranges", json!({"textDocument":{"uri":uri}}));
    assert_eq!(after["result"]["version"], 2);
    assert!(after["result"]["findings"].as_array().unwrap().is_empty());
    server.notify(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":1},"contentChanges":[{"text":source}]}),
    );
    assert_eq!(
        server.request("stateLens/ranges", json!({"textDocument":{"uri":uri}}))["result"]
            ["version"],
        2
    );
    server.notify("textDocument/didClose", json!({"textDocument":{"uri":uri}}));
    assert!(server
        .request("stateLens/ranges", json!({"textDocument":{"uri":uri}}))
        .get("error")
        .is_some());
    server.stop();
}
#[test]
fn malformed_and_oversized_messages_are_rejected() {
    assert!(read_message(&mut std::io::Cursor::new(
        b"Content-Length: 999999999\r\n\r\n"
    ))
    .is_err());
    assert!(read_message(&mut std::io::Cursor::new(b"Content-Length: 1\r\n\r\nx")).is_err());
}
#[test]
fn cli_and_server_share_findings() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/typescript/clock.ts");
    let output = Command::new(env!("CARGO_BIN_EXE_state-lens"))
        .args(["scan", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
    let uri = url::Url::from_file_path(&path).unwrap().to_string();
    let mut server = Server::start();
    server.notify("textDocument/didOpen",json!({"textDocument":{"uri":uri,"languageId":"typescript","version":1,"text":std::fs::read_to_string(path).unwrap()}}));
    let report = server.request("stateLens/ranges", json!({"textDocument":{"uri":uri}}));
    let cli_finding = &cli["documents"][0]["findings"][0];
    let editor_finding = &report["result"]["findings"][0]["finding"];
    let cli_snapshot = cli_finding["state_trace"]["snapshot_id"].as_str().unwrap();
    let editor_snapshot = editor_finding["state_trace"]["snapshot_id"]
        .as_str()
        .unwrap();
    assert_ne!(cli_snapshot, editor_snapshot);
    assert_eq!(
        serde_json::to_string(cli_finding)
            .unwrap()
            .replace(cli_snapshot, "SNAPSHOT"),
        serde_json::to_string(editor_finding)
            .unwrap()
            .replace(editor_snapshot, "SNAPSHOT")
    );
    server.stop();
}
#[test]
fn diagnostics_can_be_disabled() {
    let mut server = Server::start_with(json!({"diagnostics":false}));
    server.open_clock();
    thread::sleep(Duration::from_millis(50));
    server.request(
        "stateLens/ranges",
        json!({"textDocument":{"uri":CLOCK_URI}}),
    );
    assert!(server.notifications.iter().all(|n| n["method"] != PUBLISH));
    server.stop();
}
#[test]
fn diagnostics_track_versions_and_clear_on_close() {
    let mut server = Server::start();
    server.open_clock();
    let first = server.published(CLOCK_URI);
    assert_eq!(first["version"], 1);
    let diagnostic = &first["diagnostics"][0];
    assert_eq!(diagnostic["severity"], 3);
    assert_eq!(diagnostic["source"], "state-lens");
    assert!(diagnostic["codeDescription"]["href"]
        .as_str()
        .unwrap()
        .ends_with("/mutated-state/time-money-entropy/"));
    let start = CLOCK_SOURCE.find("Date.now()").unwrap();
    let end = start + "Date.now()".len();
    assert_eq!(
        diagnostic["range"]["start"],
        json!(position(CLOCK_SOURCE, start))
    );
    server.notify("textDocument/didChange",json!({"textDocument":{"uri":CLOCK_URI,"version":2},"contentChanges":[{"range":{"start":position(CLOCK_SOURCE,start),"end":position(CLOCK_SOURCE,end)},"text":"1"}]}));
    let second = server.published(CLOCK_URI);
    assert_eq!(second["version"], 2);
    assert!(second["diagnostics"].as_array().unwrap().is_empty());
    server.notify("textDocument/didChange",json!({"textDocument":{"uri":CLOCK_URI,"version":3},"contentChanges":[{"text":CLOCK_SOURCE}]}));
    assert_eq!(server.published(CLOCK_URI)["version"], 3);
    server.notify(
        "textDocument/didClose",
        json!({"textDocument":{"uri":CLOCK_URI}}),
    );
    assert!(server.published(CLOCK_URI)["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    server.stop();
}
#[test]
fn configuration_toggles_diagnostics() {
    let mut server = Server::start();
    server.open_clock();
    server.notify(
        "workspace/didChangeConfiguration",
        json!({"settings":{"stateLens":{"diagnostics":true}}}),
    );
    assert_eq!(
        server.published(CLOCK_URI)["diagnostics"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    server.notify(
        "workspace/didChangeConfiguration",
        json!({"settings":{"stateLens":{"diagnostics":false}}}),
    );
    assert!(server.published(CLOCK_URI)["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    server.stop();
}
#[test]
fn inlay_hints_match_annotations_and_requested_ranges() {
    let mut server = Server::start();
    server.open_clock();
    let hints = server.request(
        "textDocument/inlayHint",
        json!({"textDocument":{"uri":CLOCK_URI},"range":{"start":{"line":0,"character":0},"end":{"line":99,"character":0}}}),
    );
    let hints = hints["result"].as_array().unwrap();
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0]["label"], "◆ Mutacious");
    assert_eq!(
        hints[0]["position"],
        json!(position(
            CLOCK_SOURCE,
            CLOCK_SOURCE.find("Date.now()").unwrap()
        ))
    );
    let snapshot = server.request(
        "stateLens/ranges",
        json!({"textDocument":{"uri":CLOCK_URI}}),
    );
    assert!(snapshot["result"]["annotations"].as_array().unwrap().len() > hints.len());
    assert_eq!(hints[0]["data"]["findings"], json!([0]));
    let empty = server.request("textDocument/inlayHint", json!({"textDocument":{"uri":CLOCK_URI}, "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}));
    assert!(empty["result"].as_array().unwrap().is_empty());
    server.stop();
}

#[test]
fn hover_guide_links_respect_client_markup_support() {
    let uri = "file:///examples/mutated-state/for-loop-control-flow/example.ts";
    let url = "https://badsoftware.untra.io/en/mutated-state/time-money-entropy/";
    for (capabilities, kind, link) in [
        (
            json!({"textDocument":{"hover":{"contentFormat":["markdown","plaintext"]}}}),
            "markdown",
            format!("[Guide](<{url}>)"),
        ),
        (
            json!({"textDocument":{"hover":{"contentFormat":["plaintext"]}}}),
            "plaintext",
            format!("Guide: {url}"),
        ),
        (json!({}), "plaintext", format!("Guide: {url}")),
    ] {
        let mut server = Server::start_with_capabilities(json!({"diagnostics":true}), capabilities);
        server.notify("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"typescript","version":1,"text":CLOCK_SOURCE}}));
        let diagnostic = server.published(uri);
        assert_eq!(diagnostic["diagnostics"][0]["codeDescription"]["href"], url);
        let hover = server.request("textDocument/hover",json!({"textDocument":{"uri":uri},"position":position(CLOCK_SOURCE,CLOCK_SOURCE.find("Date.now()").unwrap())}));
        assert_eq!(hover["result"]["contents"]["kind"], kind);
        let value = hover["result"]["contents"]["value"].as_str().unwrap();
        assert_eq!(value.matches(&link).count(), 1);
        assert!(!value.contains("/for-loop-control-flow/"));
        for removed in ["Susceptibility", "Related taxonomy", "bytes", "syntax"] {
            assert!(!value.contains(removed));
            assert!(!diagnostic["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains(removed));
        }
        server.stop();
    }
}

#[test]
fn client_trace_notification_does_not_log_an_error() {
    let mut server = Server::start();
    server.notify("$/setTrace", json!({"value":"off"}));
    server.open_clock();
    server.request(
        "stateLens/ranges",
        json!({"textDocument":{"uri":CLOCK_URI}}),
    );
    assert!(!server.notifications.iter().any(|message| {
        matches!(
            message["method"].as_str(),
            Some("window/logMessage" | "window/showMessage")
        ) && matches!(message["params"]["type"].as_u64(), Some(1 | 2))
    }));
    server.stop();
}

#[test]
fn division_warning_links_and_clears_after_guard() {
    let mut server = Server::start();
    let uri = "file:///division.ts";
    server.notify("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"typescript","version":1,"text":"const d = 0; const x = 1 / d;"}}));
    let published = server.published(uri);
    let diagnostic = published["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"].as_str().unwrap().ends_with("divide-by-zero"))
        .unwrap();
    assert_eq!(diagnostic["severity"], 2);
    assert!(diagnostic["codeDescription"]["href"]
        .as_str()
        .unwrap()
        .ends_with("/unstructured-state/divide-by-zero/"));
    server.notify("textDocument/didChange", json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"const d = 0; const x = d === 0 ? null : 1 / d;"}]}));
    let published = server.published(uri);
    assert_eq!(published["version"], 2);
    assert!(published["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .all(|d| !d["code"].as_str().unwrap().ends_with("divide-by-zero")));
    server.stop();
}

#[test]
fn check_configuration_removes_and_restores_findings() {
    let mut server = Server::start();
    server.open_clock();
    for checks in [json!(["time-money-entropy"]), json!([])] {
        server.notify(
            "workspace/didChangeConfiguration",
            json!({"settings":{"stateLens":{"disabledChecks":checks}}}),
        );
        let snapshot = server.request(
            "stateLens/ranges",
            json!({"textDocument":{"uri":CLOCK_URI}}),
        );
        assert_eq!(
            snapshot["result"]["findings"]
                .as_array()
                .unwrap()
                .is_empty(),
            !checks.as_array().unwrap().is_empty()
        );
    }
    server.stop();
}
