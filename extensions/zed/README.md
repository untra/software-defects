# State Lens for Zed

Connects Zed to the `state-lens` language server for Go, Rust, and TypeScript. Zed has no arbitrary background-overlay API, so the lens appears as:

- **Hint diagnostics** on each trigger range, linking to the defect's page with evidence and call path as related information;
- **inlay hints** at each susceptible function body (`◆ mutated-state×5 (4 propagated)`);
- **hover** text on triggers.

Highlighted code is not necessarily wrong; these are susceptibility signals, not bug diagnoses.

## Install (dev extension)

```sh
make install-server        # installs ~/.local/bin/state-lens
```

In Zed, run **zed: install dev extension** and choose `extensions/zed`. Zed compiles the extension with rustup's `wasm32-wasip2` target.

The server is found on `PATH`. To use another binary, or to turn diagnostics off:

```json
{
  "lsp": {
    "state-lens": {
      "binary": { "path": "/absolute/path/to/state-lens" },
      "initialization_options": { "diagnostics": true }
    }
  },
  "inlay_hints": { "enabled": true }
}
```

If a language pins its servers, keep State Lens alongside the primary one, e.g. `"languages": { "Rust": { "language_servers": ["rust-analyzer", "state-lens", "..."] } }`.

## Debug

**zed: open log** shows extension load errors; **dev: open language server logs** shows `state-lens` traffic. After rebuilding the server, run **editor: restart language server**.
