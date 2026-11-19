## Summary

<!-- What changed and why. -->

## Checklist

- [ ] `make ci` passes locally
- [ ] Added, removed, or edited files under `examples/` or `examples-fixed/` → ran `make smoke-update` and reviewed the `tooling/state-lens/smoke.json` diff
- [ ] Changed `queries/**` → `make server` passes (runs `check-queries`)
- [ ] Changed `tooling/query-lab/**` → `make query-lab-test` passes; new Python deps are pinned in `tooling/query-lab/requirements.txt`
- [ ] Touched path handling in `build.rs` or scripts → paths are joined with `/`, never the OS separator (Windows `packages` legs)
