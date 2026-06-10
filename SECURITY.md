# Security Policy

## Supported versions

Anvil is in early development. Security fixes are applied to the latest `main` branch and the most
recent release.

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, report them privately:

- Use GitHub's [private vulnerability reporting](https://github.com/kudzaiprichard/anvil/security/advisories/new), or
- Email the maintainer at **kudzaiprichard@gmail.com**.

Please include a description of the issue, steps to reproduce, the affected platform/version, and any
relevant logs. We will acknowledge your report as soon as possible and keep you updated on the fix.

## Scope of interest

Anvil runs user-supplied code locally as part of its core feature set. We are especially interested in
reports involving:

- Sandbox escapes or ways untrusted code could affect the host beyond its intended limits.
- Unexpected filesystem, network, or process access from executed code.
- Issues in how problem packs are imported or validated.

Thank you for helping keep Anvil and its users safe.
