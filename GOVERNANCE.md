# Governance

## Model

DISTIRA follows a **maintainer-led** governance model.
Decisions are made transparently and documented in the repository.
Architecture, routing, dashboard UX, and governance changes are expected to stay aligned.

## Roles

| Role | Responsibilities |
| --- | --- |
| **Maintainers** | Merge PRs, cut releases, define roadmap priorities, and own product and security decisions |
| **Reviewers** | Review PRs, triage issues, enforce code standards, and challenge unclear routing or UX assumptions |
| **Contributors** | Submit PRs, report issues, improve documentation, and keep behavior and docs aligned |

## Decision process

1. Proposals are submitted as GitHub issues or discussions.
2. Maintainers evaluate feasibility and alignment with the roadmap.
3. Accepted proposals are tracked in `ROADMAP.md` and assigned to an iteration.
4. Behavior changes that affect routing, observability, security posture, or dashboard semantics should update the relevant docs in the same change.

## Release discipline

- `CHANGELOG.md` should record noteworthy user-visible or operator-visible changes under `[Unreleased]`.
- `README.md`, `INSTALL.md`, and community docs should not drift from runtime behavior.
- Security-sensitive changes should update `SECURITY.md` when the threat model, supported versions, or disclosure workflow changes.

## Principles

- **Clarity over cleverness** — readable code and explicit contracts.
- **Measurable optimization** — every feature must improve a quantifiable metric.
- **Sovereign-friendly defaults** — local-first, privacy-preserving by design.
- **Open and verifiable roadmap** — all plans are public in `ROADMAP.md`.
- **Truthful observability** — dashboards must distinguish upstream client metadata from routed provider and runtime facts.
- **Docs ship with product** — governance, contribution, and security files are part of the runtime contract, not afterthoughts.