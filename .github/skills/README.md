# GitHub Copilot Agent Skills

The skill directories here are **symlinks** to the canonical skills in
[`.claude/skills/`](../../.claude/skills/). They are the single source of
truth — edit the files under `.claude/skills/<name>/SKILL.md`, not these links.

GitHub Copilot discovers agent skills from `.github/skills/`, `.claude/skills/`,
or `.agents/skills/`. These symlinks make the project's skills explicit in
Copilot's native path while avoiding duplicated content.

Available skills:

- `mml2vgm-internals` — toolchain architecture and compilation pipeline
- `mml2vgm-mml-syntax` — MML syntax, dialects, and instrument definitions
- `mml2vgm-systems-emulation` — supported chips/systems and how they're emulated
