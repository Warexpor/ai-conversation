# Contributing

This is a private project. External contributions are not accepted unless arranged with the owner.

## Local development

1. Install Node 18+, Rust stable, and Tauri prerequisites.
2. `npm install`
3. `npm run tauri dev`

Before opening a PR (if invited):

```bash
npm run typecheck
npm run test:rust
```

Do not commit `.env`, API keys, or anything under `src-tauri/target/`.
