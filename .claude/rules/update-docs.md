# Rule: Update Documentation

## When to follow this rule

You finished implementing a feature. Do this before committing.

## What to update

| What changed | Where to update |
|---|---|
| New built-in or stdlib function | `docs/lang/STDLIB.md` + feature table in `CLAUDE.md` |
| New language syntax | The relevant `docs/lang/*.md` file |
| New env var or config knob | `docs/lang/CONFIGURATION.md` |
| Backlog item completed | Mark done in `docs/dev/BACKLOG.md` |
| New interpreter component | `docs/dev/INTERPRETER.md` |
| Test count changed significantly | Test table in `CLAUDE.md` |

## Rules

Write one thing per sentence. State what the feature does. Do not explain why the language exists.

Do not copy content between files. Each doc owns one topic. Link to other docs instead of repeating them.

Update `gh-pages` branch for any user-facing change. See `docs/dev/GITHUB_PAGES.md`.
