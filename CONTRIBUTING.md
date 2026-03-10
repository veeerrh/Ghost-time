# Contributing to Ghost-time

## Branching Model
| Branch | Purpose | Rule |
| :--- | :--- | :--- |
| `main` | Production-ready code only | Never commit directly. Only merge from `release/*` branches. |
| `develop` | Integration branch | All feature branches merge here. Must always pass CI. |
| `feature/xxx` | One feature at a time | e.g. `feature/window-hook`, `feature/sqlcipher-init` |
| `fix/xxx` | Bug patches | e.g. `fix/idle-detection-crash`, `fix/null-window-title` |
| `release/v0.x` | Pre-release staging | Created when a Phase ends. Bump version here. |

## Commit Message Standards
Every commit must follow **Conventional Commits**. The type prefix IS the purpose.

| Type | Example Message |
| :--- | :--- |
| `feat:` | feat(hook): log active window title + duration to stdout on every switch |
| `test:` | test(hook): verify idle detection triggers after 5 min no input |
| `fix:` | fix(hook): handle null window title on macOS screen lock gracefully |
| `style:` | style(dashboard): add responsive padding to timeline card |
| `refactor:` | refactor(classifier): move keyword rules to separate config file |
| `chore:` | chore: add sqlcipher-sys to Cargo.toml dependencies |
| `docs:` | docs: add macOS Accessibility permission setup instructions |
| `perf:` | perf(db): add composite index on timestamp + matter_id columns |

## Testing & QA Standards

| TEST | COMMAND / ACTION | EXPECTED OUTPUT |
| :--- | :--- | :--- |
| **Tauri window opens** | `cargo tauri dev` | Blank window appears, no panic in terminal |
| **Rust compiles clean** | `cargo build 2>&1 \| grep error` | Zero output (no errors) |
| **Node deps install** | `npm install && npm run build` | Exit code 0, `dist/` folder created |

## Troubleshooting

| ERROR | ROOT CAUSE | FIX |
| :--- | :--- | :--- |
| `cargo: command not found` | Rust not installed | `curl --proto=https --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| `Unresolved import tauri` | Wrong Tauri v1 vs v2 API | Check `Cargo.toml`: `tauri = '2.x'`, NOT `'1.x'` |
| `Window blank / white screen` | Vite dev server not running | Run: `npm run dev` first, then `cargo tauri dev` in separate terminal |
