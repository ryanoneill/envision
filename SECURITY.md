# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.5.x   | Yes       |
| < 0.5   | No        |

## Security Model

Envision is a TUI framework that renders to terminal emulators. Its security boundaries are:

**What envision manages:**
- Rendering styled text and widgets to the terminal via ratatui
- Processing keyboard and mouse events from crossterm
- Managing component state through The Elm Architecture (TEA) pattern

**What envision does NOT manage:**
- Network I/O, file system access, or any external resource access
- Authentication, authorization, or session management
- Encryption or cryptographic operations

Side effects (network calls, file I/O, etc.) are the responsibility of application code within `Command` callbacks, not the framework itself.

## Terminal Escape Sequences

Terminal applications face a unique attack surface: malicious content rendered to the terminal can include escape sequences that exploit vulnerabilities in terminal emulators.

### How envision mitigates this

- **Ratatui handles rendering.** All text output goes through ratatui's `Buffer` abstraction, which maps styled characters to cells rather than writing raw escape sequences. Ratatui and crossterm together handle the translation from logical cells to terminal output.
- **No raw `write!` to stdout.** Envision never writes raw bytes to the terminal directly. All rendering is mediated by ratatui's `Backend` trait.
- **CaptureBackend for testing.** The headless `CaptureBackend` never interacts with a real terminal, eliminating escape sequence concerns in test environments entirely.

### Guidance for application developers

If your application displays user-provided or external content (log files, chat messages, API responses, etc.):

1. **Sanitize before storing.** Strip or replace control characters (U+0000–U+001F, U+007F, U+0080–U+009F) from untrusted input before storing it in your application state. The TEA pattern makes this straightforward — validate in your `update` function.
2. **Limit string lengths.** Unbounded strings can cause excessive memory use or slow rendering. Truncate or paginate long content.
3. **Be cautious with ANSI in content.** If you display content that may contain ANSI escape codes (e.g., log output), consider stripping them before rendering through envision components.

## Input Validation

Envision components perform boundary validation internally:

- **Index clamping.** List, table, and tree components clamp selection indices to valid ranges. Out-of-bounds indices do not cause panics.
- **Empty collection safety.** All components handle empty data gracefully (empty lists, zero items, no tabs).
- **Resize handling.** Components adapt to terminal resize events without crashing.

For application-level input validation:

- Validate user input in your `update` function before modifying state
- Use the type system to enforce invariants (the TEA pattern's `Message` enum naturally restricts valid operations)
- The `Command` type's `try_perform_async` method routes errors to a dedicated error channel rather than panicking

## Dependency Security

Envision's runtime dependencies are:

| Dependency | Purpose |
|------------|---------|
| ratatui | Terminal rendering |
| crossterm | Terminal I/O and event handling |
| tokio | Async runtime |
| tokio-stream, tokio-util | Async stream utilities |
| async-stream, futures-util | Async combinators |
| unicode-width | Unicode character width calculation |
| compact_str | Memory-efficient string storage |
| serde, serde_json | Serialization (optional, behind `serialization` feature) |

All dependencies are well-maintained crates from the Rust ecosystem. Envision contains **zero `unsafe` code**.

## Panic Freedom

Envision is designed to avoid panics on valid input:

- No `unwrap()` on user-provided data
- Index operations use bounds checking or clamping
- Pattern matches are exhaustive
- The `Component` trait's `update` function returns `Option` rather than panicking on unhandled messages

If you discover a case where envision panics on valid input, please report it as a bug.

## Reporting a Vulnerability

If you discover a security vulnerability in envision, please report it responsibly:

1. **Do not open a public issue.** Security vulnerabilities should be reported privately.
2. **Email the maintainer** at the email address listed in the [crates.io package metadata](https://crates.io/crates/envision) or use [GitHub's private vulnerability reporting](https://github.com/ryanoneill/envision/security/advisories/new).
3. **Include details:** affected versions, reproduction steps, and potential impact.
4. **Expected response time:** We aim to acknowledge reports within 48 hours and provide a fix or mitigation plan within 7 days for confirmed vulnerabilities.

We will credit reporters in the advisory and CHANGELOG unless they prefer to remain anonymous.
