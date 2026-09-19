# Compatibility and testing plan

## Test pyramid

| Layer | Purpose | Runs on |
|---|---|---|
| Parser unit tests | Every grammar branch, casing, separators, errors | Linux + Windows |
| Alias registry tests | Completeness, uniqueness, class/namespace mapping | Linux + Windows |
| Query-builder tests | Exact WQL passed to a fake backend | Linux + Windows |
| Formatter unit tests | Exact whitespace, CRCRLF, nulls, arrays, CSV/XML/HTML escaping | Linux + Windows |
| Engine tests | Routing, node aggregation, failures, exit codes | Linux + Windows |
| Live WMI tests | COM initialization and real provider access | Windows |
| Differential tests | Same safe request against old and new executables | Windows Server 2022 |
| Oracle capture | Preserve alias/verb help and installed CLI metadata | Manual workflow |

## Next conformance slices

Each slice should add oracle fixtures and failing tests before implementation:

1. Import exact alias-specific `LIST BRIEF/FULL/STATUS` property sets from `ROOT\Cli`.
2. Capture and reproduce error text and exit codes for malformed aliases, WQL, and properties.
3. Implement object-path parsing and `ASSOC` queries.
4. Implement `CALL` using typed method metadata and harmless methods first.
5. Implement `SET`, `CREATE`, and `DELETE` with interactive confirmation tests in disposable WMI
   namespaces—not against operating-system resources.
6. Wire COM security context for remote `/NODE`, `/USER`, `/PASSWORD`, `/AUTHORITY`, `/IMPLEVEL`,
   `/AUTHLEVEL`, and `/PRIVILEGES`.
7. Reproduce `/EVERY`, `/AGGREGATE`, `/FAILFAST`, interactive mode, `/RECORD`, clipboard, output
   file encoding, and append behavior.
8. Reproduce XSL mappings, translations, locale-dependent help, and HTML/XML byte fixtures.

## Safety policy for tests

Default CI conformance cases are read-only, stable, fast, and avoid providers such as
`Win32_Product` that can be slow or trigger installer consistency work. Destructive and method-call
tests must use a disposable custom test provider or explicitly created resource and must clean up
that exact resource.

## Known gaps are assertions, not silent fallbacks

Recognized but unsupported verbs return a nonzero exit code. Custom `/FORMAT` styles are retained
in the parsed model but currently use the table renderer. These behaviors are intentionally visible
so a caller cannot mistake partial compatibility for successful emulation.
