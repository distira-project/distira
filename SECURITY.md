# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 10.18.x | Yes |
| 10.x (older patch releases) | Best effort |
| < 10.0 | No |

## Reporting a vulnerability

Please report security issues **privately** through
[GitHub Security Advisories](https://github.com/distira-project/distira/security/advisories/new).

Do **not** open a public issue for security vulnerabilities.

We aim to acknowledge reports within 48 hours and provide a fix or mitigation
within 7 days for critical issues.

## Security scope

This policy covers the live DISTIRA runtime and its operational surfaces:

- Rust backend APIs such as `/v1/compile`, `/v1/chat/completions`, `/v1/providers`, `/v1/metrics`, and `/v1/runtime/client-context`
- The MCP server in `mcp/distira-server.mjs`
- Dashboard features exposing routing, provider health, upstream lineage, alerts, audit history, and metrics export
- Provider, routing, policy, and workspace configuration under `configs/`

## Security design goals

- **Minimal context exposure:** only the compiled context reaches the LLM.
- **Policy-driven routing:** sensitive data stays on local providers.
- **Sovereign-first defaults:** on-prem deployments are first-class citizens.
- **Explicit sensitive handling:** context blocks are tagged by sensitivity level.
- **Lineage transparency:** DISTIRA distinguishes upstream client/provider/model metadata from the provider actually routed at runtime.
- **Observable operations:** provider health, alerts, runtime audit, and export surfaces are treated as security-relevant telemetry.

## Current limitations

The following are planned or expected to evolve further:

- Full secret scanning enforcement in CI and local workflows
- Production-grade authentication and authorization across all exposed routes
- Encrypted memory block persistence
- Hardened provider credential rotation
- TLS enforcement on inter-service communication