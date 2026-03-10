# Git Standards for AI Agents

You MUST strictly follow these standards when performing Git operations in this repository.

## Branching
- `main`: Never commit directly. Only merge from `release/*`.
- `develop`: Merge all feature branches here.
- `feature/xxx`: For new features.
- `fix/xxx`: For bug fixes.
- `release/v0.x`: For pre-release staging.

## Commit Messages
Use Conventional Commits format: `<type>(<scope>): <message>`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`.
