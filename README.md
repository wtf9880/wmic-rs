# wmic-rs

A native Rust reimplementation of the deprecated Windows Management Instrumentation command-line
utility (`wmic.exe`). The project is deliberately test-first: legacy WMIC is used as an oracle on a
Windows Server 2022 GitHub-hosted runner, while deterministic parsing and formatting tests run on
both Windows and Linux.

> [!IMPORTANT]
> This is an early compatibility implementation, not yet a drop-in replacement for every WMIC
> command. Read-only `GET`/`LIST` queries are functional. The parser recognizes the remaining verb
> and global-switch grammar so every missing behavior can be added behind stable tests.

## Current compatibility

| Surface | Status |
|---|---|
| All 81 inbox en-US alias names and underlying classes | Implemented |
| `PATH` and `CLASS` class queries | Implemented |
| `WHERE`, `GET`, `LIST` | Implemented for ordinary WQL queries |
| Table, value/list, CSV, basic RawXML, HForm, HTable | Implemented; exact XSL quirks still being measured |
| `/NAMESPACE`, local `/NODE`, `/OUTPUT`, `/APPEND` parsing | Implemented |
| Remote credentials, auth/impersonation, privileges | Parsed; not wired to COM yet |
| `CALL`, `SET`, `CREATE`, `DELETE`, `ASSOC` | Parsed; execution intentionally fails loudly |
| Alias-specific property lists, methods, help text, translations | Oracle capture exists; implementation pending |
| Interactive REPL, `/EVERY`, `/RECORD`, clipboard | Pending |

## Build and use

```powershell
cargo build --release
.\target\release\wmic.exe os get Caption,Version /value
.\target\release\wmic.exe process where "Name='explorer.exe'" get Name,ProcessId
.\target\release\wmic.exe path Win32_LogicalDisk get DeviceID,DriveType /format:csv
```

WMI queries only execute on Windows. The command parser, alias registry, query builder, and output
formatters are portable so their unit tests can run on any Rust development machine.

## Tests

```powershell
cargo test --all-targets
cargo build --release
.\tools\differential.ps1 -Candidate .\target\release\wmic.exe -RequireOracle
```

CI has two layers:

1. `portable-tests` exhaustively tests deterministic grammar, alias mapping, WQL construction,
   value conversion, formatting, errors, and the public API.
2. `windows-wmi` queries real WMI and runs safe differential cases against the inbox legacy
   `%WINDIR%\System32\wbem\wmic.exe` on `windows-2022`.

The manually triggered **Capture legacy WMIC oracle** workflow saves global help, every alias's
help, its `GET`/`LIST`/`CALL` help, `ROOT\Cli` alias instances, and a WBEM MOF/MFL/XSL inventory as
a workflow artifact. This matters because current Windows 11 releases no longer allow WMIC to be
added back.

See [research](docs/research.md), [architecture](docs/architecture.md), and the
[compatibility/testing plan](docs/compatibility.md).

## Design boundary

The executable calls WMI through COM using the Rust `wmi` crate. It does not invoke PowerShell,
`Get-CimInstance`, or the removed `wmic.exe`. Legacy WMIC is used only by the conformance test
harness.

## License

MIT
