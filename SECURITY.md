# Security Policy

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security issues by emailing:

**security@xlm-ns.org**

Include as much of the following as you can:

- A clear description of the vulnerability and its potential impact.
- The affected component(s): contract name, CLI command, SDK method, or
  configuration path.
- Steps to reproduce or a minimal proof-of-concept.
- Any mitigating factors you are aware of.

You will receive an acknowledgement within **2 business days**.

If you do not receive a response within 4 business days, follow up at the same
address with "REMINDER" in the subject line.

---

## Disclosure process

| Step | Owner | Target timeline |
|------|-------|----------------|
| Acknowledgement | Maintainers | 2 business days |
| Initial triage and severity rating | Maintainers | 5 business days |
| Reproduce and confirm | Maintainers | 10 business days |
| Patch development | Maintainers + reporter (optional) | Depends on severity |
| Coordinated disclosure | Both parties agree on date | ≥ 7 days after patch ships |
| Public CVE or advisory | Maintainers | Same day as disclosure |

We follow **coordinated disclosure**.  We will not disclose a vulnerability
publicly until a patch is available *and* the reporter has had a chance to
review it, unless the vulnerability is already being actively exploited.

We ask reporters to observe the same embargo until the coordinated date.

---

## Severity rating

We use a simplified four-level scale:

| Severity | Description | Example |
|----------|-------------|---------|
| **Critical** | Arbitrary fund theft or unauthorized ownership transfer on mainnet | Logic error allowing any caller to transfer a name without owner auth |
| **High** | Denial-of-service or permanent data corruption | Overflow causing a name to expire immediately on registration |
| **Medium** | Information leak or non-fund-critical contract misbehaviour | Resolver returns stale records outside the TTL window |
| **Low** | Minor issues with limited real-world impact | CLI leaks the network passphrase in a debug log |

Critical and High findings are patched and released as soon as possible.
Medium and Low findings are batched into the next scheduled release.

---

## Scope

### In scope

- All Soroban contracts under `contracts/` deployed to Stellar testnet or mainnet.
- The `xlm-ns-cli` binary and all subcommands.
- The `xlm-ns-sdk` and `xlm-ns-common` crates when used as library dependencies.
- Configuration parsing and signer key handling in the CLI.

### Out of scope

- Third-party infrastructure (Stellar core, Soroban RPC nodes, Axelar gateway).
- Vulnerabilities that require the attacker to already control the victim's
  Stellar private key.
- Issues in dependencies that are already publicly disclosed — report those
  upstream and open a non-security PR here to bump the version.
- Social-engineering attacks against maintainers.

---

## Security assumptions and threat model

The high-level trust boundaries for xlm-ns are documented in the README under
**Security and Threat Model**.  Key assumptions relevant to vulnerability reports:

1. **No privileged recovery path** — the registry and registrar have no admin
   override.  Ownership changes only through the normal expiry/grace/claimable
   flow or explicit owner-initiated transfer.  A report claiming that "the admin
   cannot reclaim a stolen name" is by design, not a bug.

2. **Auth is enforced by Soroban** — every mutation requires
   `Address::require_auth()`.  A finding that bypasses this is Critical.

3. **Signer secrets are never stored in structs** — the CLI reads secrets from
   environment variables at signing time only.  A finding that causes secrets to
   be written to disk, logs, or network is High.

4. **The SDK is currently mock-based** — it does not make live RPC calls.
   Security findings against the mock layer are Low unless they affect the
   real-client path when it ships.

---

## Preferred languages and tools

We can receive reports in English.  PoC code in Rust, shell, or Python is
preferred.

---

## Recognition

We do not currently run a paid bug-bounty programme.  Reporters of confirmed
High or Critical vulnerabilities will be credited in the release notes and
security advisory (with their consent) and offered a spot on the project's
acknowledgements page.

---

## Supported versions

| Version | Supported |
|---------|-----------|
| Latest on `main` | Yes |
| Tagged releases | Current + previous minor only |
| Older releases | No — please upgrade |
