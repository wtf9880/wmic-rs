use crate::aliases;
use crate::cli::{Command, GlobalOptions, HelpTopic, Invocation, ListMode, Query, Verb};
use crate::output;
use std::collections::BTreeMap;
use std::fmt;

pub type Record = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => Ok(()),
            Self::String(value) | Self::Object(value) => f.write_str(value),
            Self::Signed(value) => write!(f, "{value}"),
            Self::Unsigned(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => f.write_str(if *value { "TRUE" } else { "FALSE" }),
            Self::Array(values) => {
                f.write_str("{")?;
                for (index, value) in values.iter().enumerate() {
                    if index != 0 { f.write_str("; ")?; }
                    write!(f, "{value}")?;
                }
                f.write_str("}")
            }
        }
    }
}

pub trait Backend {
    fn query(&self, namespace: &str, node: &str, wql: &str) -> Result<Vec<Record>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl Execution {
    fn success(stdout: String) -> Self {
        Self { stdout, stderr: String::new(), exit_code: 0 }
    }

    fn error(stderr: String) -> Self {
        Self { stdout: String::new(), stderr, exit_code: 1 }
    }
}

pub fn execute(invocation: &Invocation, backend: &dyn Backend) -> Execution {
    match &invocation.command {
        Command::Help(HelpTopic::Global) => Execution::success(aliases::global_help()),
        Command::Help(HelpTopic::Alias(name)) => {
            let Some(alias) = aliases::find(name) else {
                return Execution::error(format!("Alias not found: {name}\r\n"));
            };
            Execution::success(aliases::alias_help(alias))
        }
        Command::Help(HelpTopic::Command(name)) => Execution::success(command_help(name)),
        Command::Help(HelpTopic::Switch(name)) => Execution::success(switch_help(name)),
        Command::Context => Execution::success(context(&invocation.globals)),
        Command::Exit => Execution::success(String::new()),
        Command::Query(query) => execute_query(query, &invocation.globals, backend),
    }
}

fn execute_query(query: &Query, globals: &GlobalOptions, backend: &dyn Backend) -> Execution {
    if !matches!(query.verb, Verb::Get(_) | Verb::List(_)) {
        return Execution::error(format!(
            "ERROR:\r\nDescription = Verb {:?} is parsed but not implemented yet\r\n",
            query.verb
        ));
    }
    let properties = projected_properties(query);
    let wql = build_wql(query, &properties);
    let mut all_records = Vec::new();
    for node in &globals.nodes {
        match backend.query(&query.source.namespace, node, &wql) {
            Ok(mut records) => all_records.append(&mut records),
            Err(error) => {
                return Execution::error(format!("Node - {node}\r\nERROR:\r\nDescription = {error}\r\n"));
            }
        }
    }
    let requested_node = globals.nodes.first().map(String::as_str).unwrap_or("localhost");
    let local_name;
    let node = if requested_node.eq_ignore_ascii_case("localhost") || requested_node == "." {
        local_name = std::env::var("COMPUTERNAME")
            .unwrap_or_else(|_| requested_node.into())
            .to_ascii_lowercase();
        &local_name
    } else {
        requested_node
    };
    Execution::success(output::render(&all_records, &properties, &query.format, node))
}

pub fn projected_properties(query: &Query) -> Vec<String> {
    match &query.verb {
        Verb::Get(properties) => properties.clone(),
        Verb::List(ListMode::Instance) => vec!["__PATH".into()],
        Verb::List(ListMode::System) => vec![
            "__CLASS".into(), "__DERIVATION".into(), "__DYNASTY".into(), "__GENUS".into(),
            "__NAMESPACE".into(), "__PATH".into(), "__PROPERTY_COUNT".into(),
            "__RELPATH".into(), "__SERVER".into(), "__SUPERCLASS".into(),
        ],
        Verb::List(ListMode::Status) => vec!["Status".into()],
        // Alias-specific BRIEF/FULL property sets are populated from ROOT\Cli in a future phase.
        Verb::List(ListMode::Brief | ListMode::Full) => vec!["*".into()],
        _ => Vec::new(),
    }
}

pub fn build_wql(query: &Query, properties: &[String]) -> String {
    let projection = if properties.is_empty() { "*".into() } else { properties.join(",") };
    let mut wql = format!("SELECT {projection} FROM {}", query.source.class);
    if let Some(filter) = &query.filter {
        wql.push_str(" WHERE ");
        wql.push_str(filter);
    }
    wql
}

fn context(options: &GlobalOptions) -> String {
    format!(
        "NAMESPACE    : {}\nROLE         : {}\nNODE(S)      : {}\nIMPLEVEL     : {}\nAUTHLEVEL    : {}\nLOCALE       : {}\nPRIVILEGES   : {}\nTRACE        : {}\nRECORD       : {}\nINTERACTIVE  : {}\nFAILFAST     : {:?}\nOUTPUT       : {:?}\nAGGREGATE    : {}\nAUTHORITY    : {}\n",
        options.namespace,
        options.role,
        options.nodes.join(","),
        options.impersonation,
        options.authentication,
        options.locale,
        on_off(options.privileges),
        on_off(options.trace),
        options.record.as_deref().unwrap_or("OFF"),
        on_off(options.interactive),
        options.fail_fast,
        options.destination,
        on_off(options.aggregate),
        options.authority.as_deref().unwrap_or("N/A"),
    )
}

fn on_off(value: bool) -> &'static str {
    if value { "ON" } else { "OFF" }
}

fn command_help(name: &str) -> String {
    match name.to_ascii_uppercase().as_str() {
        "PATH" => "PATH <WMI class or object path> [WHERE <condition>] <verb>\n".into(),
        "CLASS" => "CLASS <WMI class> [WHERE <condition>] <verb>\n".into(),
        "CONTEXT" => "CONTEXT - Displays the state of all global switches.\n".into(),
        other => format!("{other} /?\n"),
    }
}

fn switch_help(name: &str) -> String {
    format!("/{name} - global switch help is not yet captured for this locale.\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parse;
    use std::cell::RefCell;

    #[derive(Default)]
    struct FakeBackend {
        calls: RefCell<Vec<(String, String, String)>>,
        result: Vec<Record>,
    }

    impl Backend for FakeBackend {
        fn query(&self, namespace: &str, node: &str, wql: &str) -> Result<Vec<Record>, String> {
            self.calls.borrow_mut().push((namespace.into(), node.into(), wql.into()));
            Ok(self.result.clone())
        }
    }

    #[test]
    fn get_builds_wql_without_rewriting_user_filter() {
        let invocation = parse(["process", "where", "Name='cmd.exe'", "get", "Name,ProcessId", "/value"]).unwrap();
        let backend = FakeBackend::default();
        let execution = execute(&invocation, &backend);
        assert_eq!(execution.exit_code, 0);
        assert_eq!(backend.calls.borrow()[0].2, "SELECT Name,ProcessId FROM Win32_Process WHERE Name='cmd.exe'");
    }

    #[test]
    fn namespace_and_nodes_reach_backend() {
        let invocation = parse(["/namespace:root\\wmi", "/node:a,b", "path", "Some_Class", "get", "Name"]).unwrap();
        let backend = FakeBackend::default();
        execute(&invocation, &backend);
        let calls = backend.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], (r"root\wmi".into(), "a".into(), "SELECT Name FROM Some_Class".into()));
        assert_eq!(calls[1].1, "b");
    }

    #[test]
    fn unsupported_mutation_fails_loudly() {
        let invocation = parse(["process", "where", "ProcessId=1", "delete"]).unwrap();
        let execution = execute(&invocation, &FakeBackend::default());
        assert_eq!(execution.exit_code, 1);
        assert!(execution.stderr.contains("parsed but not implemented"));
    }
}
