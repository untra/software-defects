# State Lens for VS Code

A green background lens for mutated-state susceptibility in Go, Rust, and TypeScript. Highlighted code is not necessarily wrong. Local mutation and observable effects remain distinct; only supported observable effects propagate through resolved calls.

## Install locally

From the repository root:

```sh
make install-server install-vscode
```

This installs `state-lens` into `~/.local/bin` and the packaged `.vsix` into VS Code. The extension resolves the server from `stateLens.serverPath`, then the `STATE_LENS_SERVER` environment variable, then `PATH`. The extension does not download or execute project code and does not upload source. It runs only in trusted workspaces; for remote workspaces, install the binary in the remote environment.

For iteration, open the repository in VS Code and launch **State Lens: Extension Development Host**. It builds the release server, points `STATE_LENS_SERVER` at it, and prompts for a folder to open.

## Use

Open a `.go`, `.rs`, or `.ts` file. A light green function body indicates susceptibility; stronger green marks an operation. Propagated effects have a lighter background. Hover an operation or function body to see the reason, resolution level, and originating evidence. Use **Read the defect guide ↗** to open the finding’s English guide on badsoftware.untra.io. The Explorer's **State Lens Evidence** view navigates to operations.

- **State Lens: Toggle Susceptibility Lens** enables/disables decoration.
- **State Lens: Refresh Analysis** requests a new snapshot.
- `stateLens.directOnly` hides propagated effects.
- `stateLens.opacity` adjusts background intensity.
- `stateLens.accessibleMarkers` adds borders; text descriptions are always available.
- `stateLens.diagnostics` (off by default) also publishes findings as Hint diagnostics for Problems, agents, and other LSP consumers.

Functions with findings also get an inlay summary such as `◆ mutated-state×3 (2 propagated)`.

The status bar reports partial analysis when syntax errors, unresolved calls, or background indexing remain. Changes clear old decorations immediately; only current document snapshots are displayed. Unless `stateLens.diagnostics` is enabled, no susceptibility messages are placed in Problems. Changing the server executable requires reloading the extension host.

## Limits

This release includes the mutated-state lens only. It has no validated bug diagnostics, general type inference, dynamic-dispatch resolution, full alias analysis, or macro expansion. Related CWEs are taxonomy references, not vulnerability claims. TSX/JavaScript are not included in this first grammar pack.

See [the analyzer documentation](https://github.com/untra/software-defects/tree/main/tooling/state-lens) for supported resolution, scoring, coverage, and planned blue/red lenses.
