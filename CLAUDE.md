# CLAUDE.md

## gstack

Use the `/browse` skill from gstack for all web browsing. Never use `mcp__claude-in-chrome__*` tools.

### Available skills

- `/office-hours` - Office hours
- `/plan-ceo-review` - Plan CEO review
- `/plan-eng-review` - Plan engineering review
- `/plan-design-review` - Plan design review
- `/design-consultation` - Design consultation
- `/design-shotgun` - Design shotgun
- `/design-html` - Design HTML
- `/review` - Code review
- `/ship` - Ship
- `/land-and-deploy` - Land and deploy
- `/canary` - Canary
- `/benchmark` - Benchmark
- `/browse` - Web browsing
- `/connect-chrome` - Connect Chrome
- `/qa` - QA
- `/qa-only` - QA only
- `/design-review` - Design review
- `/setup-browser-cookies` - Setup browser cookies
- `/setup-deploy` - Setup deploy
- `/retro` - Retro
- `/investigate` - Investigate
- `/document-release` - Document release
- `/codex` - Codex
- `/cso` - CSO
- `/autoplan` - Autoplan
- `/plan-devex-review` - Plan devex review
- `/devex-review` - Devex review
- `/careful` - Careful mode
- `/freeze` - Freeze
- `/guard` - Guard
- `/unfreeze` - Unfreeze
- `/gstack-upgrade` - Upgrade gstack
- `/learn` - Learn

## Release Process
All commits to `main` use the message format `release: X.Y.Z`.
Deployment is triggered by pushing a `v*` tag:
1. `git commit -m "release: X.Y.Z"`
2. `git push origin main`
3. `git tag vX.Y.Z && git push origin vX.Y.Z`

The tag push triggers `.github/workflows/release.yml` which builds Tauri desktop apps for macOS/Linux/Windows and creates a GitHub Release with artifacts.

## Design System
Always read DESIGN.md before making any visual or UI decisions.
All font choices, colors, spacing, and aesthetic direction are defined there.
Do not deviate without explicit user approval.
In QA mode, flag any code that doesn't match DESIGN.md.

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke office-hours
- Bugs, errors, "why is this broken", 500 errors → invoke investigate
- Ship, deploy, push, create PR → invoke ship
- QA, test the site, find bugs → invoke qa
- Code review, check my diff → invoke review
- Update docs after shipping → invoke document-release
- Weekly retro → invoke retro
- Design system, brand → invoke design-consultation
- Visual audit, design polish → invoke design-review
- Architecture review → invoke plan-eng-review
- Save progress, checkpoint, resume → invoke checkpoint
- Code quality, health check → invoke health
