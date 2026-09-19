# WMIC research notes

This document records the facts the implementation and test strategy rely on. It intentionally
separates documented behavior from behavior that must be measured against an original binary.

## Supported platform versus removed frontend

Microsoft deprecated the WMIC utility in Windows 10 21H1, but explicitly states that the WMI API
itself is not affected. As of August 2026, Windows 11 24H2 and 25H2 no longer contain WMIC and the
Feature on Demand cannot be added back. Therefore this implementation uses the supported WMI COM
API, while the test oracle must run on an OS image that still contains the old frontend.

Sources:

- [WMIC utility reference](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmic)
- [Removed Windows client features](https://learn.microsoft.com/en-us/windows/whats-new/removed-features)
- [COM API for WMI](https://learn.microsoft.com/en-us/windows/win32/wmisdk/com-api-for-wmi)
- [`IWbemServices::ExecQuery`](https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execquery)

## Grammar discovered from Microsoft documentation

WMIC has four layers rather than a conventional subcommand parser:

1. Global switches establish a context (`/NAMESPACE`, `/ROLE`, `/NODE`, security, locale,
   recording, redirection, interaction, fail-fast, and aggregation).
2. An alias resolves a friendly name to a WMI class and carries alias-specific formats, property
   sets, and method metadata. `PATH` and `CLASS` escape alias mode.
3. `WHERE` selects instances.
4. A verb performs `ASSOC`, `CALL`, `CREATE`, `DELETE`, `GET`, `LIST`, or `SET`.

`LIST` has `BRIEF`, `FULL`, `INSTANCE`, `STATUS`, and `SYSTEM` forms. `GET` and `LIST` accept
`/TRANSLATE`, `/EVERY`, and `/FORMAT`; `GET` additionally accepts `/VALUE` and `/ALL`. `ASSOC`
accepts `/RESULTCLASS`, `/RESULTROLE`, and `/ASSOCCLASS`.

Sources:

- [WMIC aliases](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmic#alias)
- [WMIC verbs](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmic#verbs)
- [WMIC switches](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmic#switches)
- [WMI Query Language](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmi-query-language)

## Alias and formatter metadata

The legacy frontend's behavior is partly data-driven. Windows registers CLI alias metadata in the
`ROOT\Cli` WMI namespace and ships formatting/transformation resources under
`%WINDIR%\System32\wbem`. This is why a truly compatible implementation cannot be derived from the
top-level help page alone: alias-specific `LIST BRIEF`, method arguments, value translations,
localized help, and XSL formatting must be inventoried.

The `capture-wmic-oracle.yml` workflow preserves:

- global help;
- `/?`, `GET /?`, `LIST /?`, and `CALL /?` output for all 81 aliases;
- serialized `MSFT_CliAlias` instances when available;
- names, sizes, and hashes for installed MOF, MFL, and XSL files;
- the exact legacy binary version and SHA-256.

No Microsoft binaries or copyrighted system files are committed to this repository.

## Rust WMI binding

The project uses `wmi` 0.18, which queries arbitrary classes into
`HashMap<String, Variant>` and exposes lower-level method APIs. It is backed by the Microsoft
`windows` crate and the native WMI COM interfaces.

- [`wmi-rs` source and examples](https://github.com/ohadravid/wmi-rs)
- [`wmi` crate documentation](https://docs.rs/wmi/latest/wmi/)
- [Microsoft `windows` crate `IWbemServices` bindings](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Wmi/struct.IWbemServices.html)

## What “exact” means here

Compatibility is evaluated along independent axes:

- accepted spellings, case folding, quoting, and ordering;
- selected WMI namespace/class/instances/properties;
- side effects and confirmation behavior;
- exit code, stdout, and stderr routing;
- textual values, column ordering/padding, escaping, line endings, and encoding;
- remote node, authentication, impersonation, locale, and privilege semantics;
- interactive state and repeated-query behavior.

The current live comparison normalizes only encoding, the historical doubled-carriage-return
line ending, and trailing column padding. Exact byte fixtures belong in deterministic tests once
captured for a pinned legacy binary and locale.
