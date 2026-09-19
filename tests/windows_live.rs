#![cfg(windows)]

use wmic_rs::windows_backend::WindowsWmi;
use wmic_rs::{execute, parse};

#[test]
fn queries_real_operating_system_provider() {
    let result = execute(
        &parse(["path", "Win32_OperatingSystem", "get", "OSLanguage", "/value"]).unwrap(),
        &WindowsWmi,
    );
    assert_eq!(result.exit_code, 0, "{}", result.stderr);
    assert!(result.stdout.contains("OSLanguage="));
}

#[test]
fn every_inbox_alias_at_least_resolves_to_a_declared_class() {
    // Querying every provider would be slow and some hardware/provider classes legitimately do
    // not exist on a VM. This validates the mapping; the oracle-capture workflow inventories
    // provider availability and alias-specific behavior separately.
    for alias in wmic_rs::aliases::ALIASES {
        let invocation = parse([alias.name, "get", "__CLASS", "/value"])
            .unwrap_or_else(|error| panic!("{}: {error}", alias.name));
        let wmic_rs::Command::Query(query) = invocation.command else { panic!() };
        assert_eq!(query.source.class, alias.class);
    }
}
