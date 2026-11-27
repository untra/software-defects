#!/usr/bin/env python3
"""Reproducible CLI and warm edit-to-range latency measurements."""
import argparse
import json
import platform
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

SAMPLES = 5
WORKSPACE_FILES = 100


def request(process, method, params, identifier=None):
    message = {"jsonrpc": "2.0", "method": method, "params": params}
    if identifier is not None:
        message["id"] = identifier
    payload = json.dumps(message).encode()
    process.stdin.write(f"Content-Length: {len(payload)}\r\n\r\n".encode() + payload)
    process.stdin.flush()
    if identifier is None:
        return None
    while True:
        headers = {}
        while (line := process.stdout.readline()) not in (b"\r\n", b"\n"):
            if not line:
                raise RuntimeError("Server exited")
            key, value = line.decode().split(":", 1)
            headers[key.lower()] = value.strip()
        response = json.loads(process.stdout.read(int(headers["content-length"])))
        if response.get("id") == identifier:
            if "error" in response:
                raise RuntimeError(response["error"])
            return response["result"]


def summary(values):
    return {"samples": len(values), "median_ms": round(statistics.median(values), 2), "max_ms": round(max(values), 2)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix="state-lens-benchmark-") as directory:
        root = Path(directory)
        for index in range(WORKSPACE_FILES):
            (root / f"module{index}.ts").write_text(f"export function timestamp{index}() {{ return Date.now(); }}\n")
        scans = []
        for _ in range(SAMPLES):
            start = time.perf_counter()
            subprocess.run([str(binary), "scan", str(root)], check=True, stdout=subprocess.DEVNULL)
            scans.append((time.perf_counter() - start) * 1000)
        process = subprocess.Popen([str(binary), "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        try:
            request(process, "initialize", {}, 1)
            request(process, "initialized", {})
            for index in range(WORKSPACE_FILES):
                file = root / f"module{index}.ts"
                request(process, "textDocument/didOpen", {"textDocument": {"uri": file.as_uri(), "languageId": "typescript", "version": 1, "text": file.read_text()}})
            uri = (root / "module0.ts").as_uri()
            request(process, "stateLens/ranges", {"textDocument": {"uri": uri}}, 2)
            edits = []
            for index in range(SAMPLES):
                start = time.perf_counter()
                request(process, "textDocument/didChange", {"textDocument": {"uri": uri, "version": index + 2}, "contentChanges": [{"text": f"export function timestamp0() {{ return Date.now() + {index}; }}\n"}]})
                request(process, "stateLens/ranges", {"textDocument": {"uri": uri}}, index + 3)
                edits.append((time.perf_counter() - start) * 1000)
            request(process, "shutdown", None, SAMPLES + 4)
            request(process, "exit", None)
            process.wait(timeout=10)
        finally:
            if process.poll() is None:
                process.kill()
            process.wait()
        print(json.dumps({"platform": platform.platform(), "architecture": platform.machine(), "workspace_files": WORKSPACE_FILES, "fixture": "100 small TypeScript files with independent clock calls", "cli_scan": summary(scans), "warm_edit_to_ranges": summary(edits), "editor_debounce_ms": 180, "notes": "Synthetic baseline; excludes VS Code rendering and does not establish performance on large call graphs."}, indent=2))


if __name__ == "__main__":
    main()
