use std::collections::BTreeMap;
use wmic_rs::{Backend, Record, Value, execute, parse};

struct FixedBackend;

impl Backend for FixedBackend {
    fn query(&self, _: &str, _: &str, _: &str) -> Result<Vec<Record>, String> {
        Ok(vec![BTreeMap::from([
            ("Caption".into(), Value::String("Test Windows".into())),
            ("OSLanguage".into(), Value::Unsigned(1033)),
        ])])
    }
}

#[test]
fn library_consumers_can_parse_execute_and_render() {
    let invocation = parse(["os", "get", "Caption,OSLanguage", "/value"]).unwrap();
    let result = execute(&invocation, &FixedBackend);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "\r\r\nCaption=Test Windows\r\r\nOSLanguage=1033\r\r\n\r\r\n");
}

#[test]
fn backend_errors_are_reported_with_wmic_style_sections() {
    struct Broken;
    impl Backend for Broken {
        fn query(&self, _: &str, _: &str, _: &str) -> Result<Vec<Record>, String> {
            Err("Invalid class".into())
        }
    }
    let result = execute(&parse(["path", "Not_A_Class", "get", "Name"]).unwrap(), &Broken);
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("ERROR:\r\nDescription = Invalid class"));
}
