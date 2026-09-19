use crate::aliases;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalOptions {
    pub namespace: String,
    pub role: String,
    pub nodes: Vec<String>,
    pub impersonation: String,
    pub authentication: String,
    pub locale: String,
    pub privileges: bool,
    pub trace: bool,
    pub record: Option<String>,
    pub interactive: bool,
    pub fail_fast: FailFast,
    pub user: Option<String>,
    pub password: Option<String>,
    pub authority: Option<String>,
    pub aggregate: bool,
    pub destination: Destination,
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            namespace: r"root\cimv2".into(),
            role: r"root\cli".into(),
            nodes: vec!["localhost".into()],
            impersonation: "Impersonate".into(),
            authentication: "Packet".into(),
            locale: "ms_409".into(),
            privileges: false,
            trace: false,
            record: None,
            interactive: true,
            fail_fast: FailFast::Off,
            user: None,
            password: None,
            authority: None,
            aggregate: true,
            destination: Destination::Stdout,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailFast {
    On,
    Off,
    Timeout(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    Stdout,
    Clipboard { append: bool },
    File { path: String, append: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub globals: GlobalOptions,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help(HelpTopic),
    Context,
    Exit,
    Query(Box<Query>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpTopic {
    Global,
    Alias(String),
    Command(String),
    Switch(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub source: Source,
    pub filter: Option<String>,
    pub verb: Verb,
    pub format: OutputFormat,
    pub every: Option<u64>,
    pub translate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub spelling: String,
    pub class: String,
    pub namespace: String,
    pub kind: SourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Alias,
    Path,
    Class,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    Get(Vec<String>),
    List(ListMode),
    Call { method: String, arguments: String },
    Set(String),
    Create(String),
    Delete,
    Assoc(AssocOptions),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMode {
    Brief,
    Full,
    Instance,
    Status,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssocOptions {
    pub result_class: Option<String>,
    pub result_role: Option<String>,
    pub assoc_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Value,
    Csv,
    List,
    RawXml,
    HForm,
    HTable,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn parse<I, S>(arguments: I) -> Result<Invocation, ParseError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = arguments.into_iter().map(Into::into).collect();
    let mut globals = GlobalOptions::default();
    let mut cursor = 0;

    while cursor < args.len() {
        let token = &args[cursor];
        if is_help(token) {
            return Ok(Invocation { globals, command: Command::Help(HelpTopic::Global) });
        }
        if !is_global_switch(token) {
            break;
        }
        if args.get(cursor + 1).is_some_and(|value| is_help(value))
            && split_switch(token).1.is_none()
        {
            return Ok(Invocation {
                globals,
                command: Command::Help(HelpTopic::Switch(
                    split_switch(token).0.to_ascii_uppercase(),
                )),
            });
        }
        apply_global_switch(&mut globals, token)?;
        cursor += 1;
    }

    if cursor == args.len() {
        return Ok(Invocation { globals, command: Command::Help(HelpTopic::Global) });
    }

    let head = &args[cursor];
    cursor += 1;

    if head.eq_ignore_ascii_case("context") {
        return topic_or_command(&args[cursor..], globals, "CONTEXT", Command::Context);
    }
    if head.eq_ignore_ascii_case("quit") || head.eq_ignore_ascii_case("exit") {
        return Ok(Invocation { globals, command: Command::Exit });
    }
    if head.starts_with('/') {
        return Err(ParseError(format!("Invalid global switch: {head}")));
    }

    let (source, rest) = if head.eq_ignore_ascii_case("path")
        || head.eq_ignore_ascii_case("class")
    {
        if cursor == args.len() || is_help(&args[cursor]) {
            return Ok(Invocation {
                globals,
                command: Command::Help(HelpTopic::Command(head.to_ascii_uppercase())),
            });
        }
        let spelling = args[cursor].clone();
        cursor += 1;
        let kind = if head.eq_ignore_ascii_case("path") {
            SourceKind::Path
        } else {
            SourceKind::Class
        };
        (
            Source {
                spelling: spelling.clone(),
                class: spelling,
                namespace: globals.namespace.clone(),
                kind,
            },
            &args[cursor..],
        )
    } else if let Some(alias) = aliases::find(head) {
        if cursor < args.len() && is_help(&args[cursor]) {
            return Ok(Invocation {
                globals,
                command: Command::Help(HelpTopic::Alias(alias.name.into())),
            });
        }
        (
            Source {
                spelling: alias.name.into(),
                class: alias.class.into(),
                namespace: if globals.namespace.eq_ignore_ascii_case(r"root\cimv2") {
                    alias.namespace.into()
                } else {
                    globals.namespace.clone()
                },
                kind: SourceKind::Alias,
            },
            &args[cursor..],
        )
    } else {
        return Err(ParseError(format!("Alias not found: {head}")));
    };

    let query = parse_query(source, rest)?;
    Ok(Invocation { globals, command: Command::Query(Box::new(query)) })
}

fn topic_or_command(
    rest: &[String],
    globals: GlobalOptions,
    name: &str,
    command: Command,
) -> Result<Invocation, ParseError> {
    if rest.is_empty() {
        Ok(Invocation { globals, command })
    } else if rest.len() == 1 && is_help(&rest[0]) {
        Ok(Invocation {
            globals,
            command: Command::Help(HelpTopic::Command(name.into())),
        })
    } else {
        Err(ParseError(format!("Unexpected argument after {name}: {}", rest[0])))
    }
}

fn parse_query(source: Source, args: &[String]) -> Result<Query, ParseError> {
    let mut cursor = 0;
    let mut filter = None;
    if args.get(cursor).is_some_and(|s| s.eq_ignore_ascii_case("where")) {
        cursor += 1;
        let start = cursor;
        while cursor < args.len() && !is_verb(&args[cursor]) && !is_query_switch(&args[cursor]) {
            cursor += 1;
        }
        if cursor == start {
            return Err(ParseError("WHERE requires a condition".into()));
        }
        filter = Some(args[start..cursor].join(" "));
    }

    let mut verb = Verb::List(ListMode::Full);
    if let Some(token) = args.get(cursor) {
        if token.eq_ignore_ascii_case("get") {
            cursor += 1;
            let mut properties = Vec::new();
            while cursor < args.len() && !is_query_switch(&args[cursor]) {
                properties.extend(
                    args[cursor]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(ToOwned::to_owned),
                );
                cursor += 1;
            }
            if properties.is_empty() {
                return Err(ParseError("GET requires at least one property".into()));
            }
            verb = Verb::Get(properties);
        } else if token.eq_ignore_ascii_case("list") {
            cursor += 1;
            let mode = args
                .get(cursor)
                .and_then(|s| parse_list_mode(s))
                .unwrap_or(ListMode::Full);
            if args.get(cursor).and_then(|s| parse_list_mode(s)).is_some() {
                cursor += 1;
            }
            verb = Verb::List(mode);
        } else if token.eq_ignore_ascii_case("delete") {
            cursor += 1;
            verb = Verb::Delete;
        } else if token.eq_ignore_ascii_case("set") || token.eq_ignore_ascii_case("create") {
            let create = token.eq_ignore_ascii_case("create");
            cursor += 1;
            let start = cursor;
            while cursor < args.len() && !is_query_switch(&args[cursor]) {
                cursor += 1;
            }
            if cursor == start {
                return Err(ParseError(if create {
                    "CREATE requires property assignments".into()
                } else {
                    "SET requires property assignments".into()
                }));
            }
            let assignments = args[start..cursor].join(" ");
            verb = if create { Verb::Create(assignments) } else { Verb::Set(assignments) };
        } else if token.eq_ignore_ascii_case("call") {
            cursor += 1;
            let method = args
                .get(cursor)
                .ok_or_else(|| ParseError("CALL requires a method".into()))?
                .clone();
            cursor += 1;
            let start = cursor;
            while cursor < args.len() && !is_query_switch(&args[cursor]) {
                cursor += 1;
            }
            verb = Verb::Call { method, arguments: args[start..cursor].join(" ") };
        } else if token.eq_ignore_ascii_case("assoc") {
            cursor += 1;
            verb = Verb::Assoc(AssocOptions::default());
        } else if !is_query_switch(token) {
            return Err(ParseError(format!("Invalid verb: {token}")));
        }
    }

    let mut format = default_format(&verb);
    let mut every = None;
    let mut translate = None;
    let mut assoc = match &verb {
        Verb::Assoc(value) => Some(value.clone()),
        _ => None,
    };
    while cursor < args.len() {
        let (name, value) = split_switch(&args[cursor]);
        match name.to_ascii_uppercase().as_str() {
            "VALUE" => format = OutputFormat::Value,
            "ALL" => format = OutputFormat::Table,
            "FORMAT" => {
                let value = require_value("FORMAT", value)?;
                format = parse_format(value);
            }
            "EVERY" => {
                let value = require_value("EVERY", value)?;
                every = Some(value.parse().map_err(|_| {
                    ParseError(format!("Invalid /EVERY interval: {value}"))
                })?);
            }
            "TRANSLATE" => translate = Some(require_value("TRANSLATE", value)?.into()),
            "RESULTCLASS" | "RESULTROLE" | "ASSOCCLASS" if assoc.is_some() => {
                let value = require_value(name, value)?.to_owned();
                let options = assoc.as_mut().expect("checked above");
                match name.to_ascii_uppercase().as_str() {
                    "RESULTCLASS" => options.result_class = Some(value),
                    "RESULTROLE" => options.result_role = Some(value),
                    _ => options.assoc_class = Some(value),
                }
            }
            _ => return Err(ParseError(format!("Invalid query switch: {}", args[cursor]))),
        }
        cursor += 1;
    }
    if let Some(options) = assoc {
        verb = Verb::Assoc(options);
    }

    Ok(Query { source, filter, verb, format, every, translate })
}

fn default_format(verb: &Verb) -> OutputFormat {
    match verb {
        Verb::List(_) => OutputFormat::List,
        Verb::Assoc(_) => OutputFormat::HTable,
        _ => OutputFormat::Table,
    }
}

fn parse_format(value: &str) -> OutputFormat {
    match value.to_ascii_lowercase().as_str() {
        "table" => OutputFormat::Table,
        "value" | "list" => OutputFormat::Value,
        "csv" => OutputFormat::Csv,
        "rawxml" => OutputFormat::RawXml,
        "hform" => OutputFormat::HForm,
        "htable" => OutputFormat::HTable,
        _ => OutputFormat::Custom(value.into()),
    }
}

fn parse_list_mode(value: &str) -> Option<ListMode> {
    Some(match value.to_ascii_uppercase().as_str() {
        "BRIEF" => ListMode::Brief,
        "FULL" => ListMode::Full,
        "INSTANCE" => ListMode::Instance,
        "STATUS" => ListMode::Status,
        "SYSTEM" => ListMode::System,
        _ => return None,
    })
}

fn is_help(value: &str) -> bool {
    value == "/?"
        || value.eq_ignore_ascii_case("/?brief")
        || value.eq_ignore_ascii_case("/?full")
        || value.eq_ignore_ascii_case("/?:brief")
        || value.eq_ignore_ascii_case("/?:full")
}

fn is_verb(value: &str) -> bool {
    ["get", "list", "call", "set", "create", "delete", "assoc"]
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn is_query_switch(value: &str) -> bool {
    let (name, _) = split_switch(value);
    ["VALUE", "ALL", "FORMAT", "EVERY", "TRANSLATE", "RESULTCLASS", "RESULTROLE", "ASSOCCLASS"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn is_global_switch(value: &str) -> bool {
    let (name, _) = split_switch(value);
    [
        "NAMESPACE", "ROLE", "NODE", "IMPLEVEL", "AUTHLEVEL", "LOCALE", "PRIVILEGES",
        "TRACE", "RECORD", "INTERACTIVE", "FAILFAST", "USER", "PASSWORD", "OUTPUT",
        "APPEND", "AGGREGATE", "AUTHORITY",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn split_switch(value: &str) -> (&str, Option<&str>) {
    let value = value.strip_prefix('/').unwrap_or(value);
    value.split_once(':').map_or((value, None), |(name, value)| (name, Some(value)))
}

fn require_value<'a>(name: &str, value: Option<&'a str>) -> Result<&'a str, ParseError> {
    value.filter(|s| !s.is_empty()).ok_or_else(|| ParseError(format!("/{name} requires a value")))
}

fn on_off(name: &str, value: Option<&str>) -> Result<bool, ParseError> {
    match require_value(name, value)?.to_ascii_lowercase().as_str() {
        "on" | "enable" | "true" => Ok(true),
        "off" | "disable" | "false" => Ok(false),
        other => Err(ParseError(format!("Invalid /{name} value: {other}"))),
    }
}

fn apply_global_switch(options: &mut GlobalOptions, token: &str) -> Result<(), ParseError> {
    let (name, value) = split_switch(token);
    match name.to_ascii_uppercase().as_str() {
        "NAMESPACE" => options.namespace = require_value(name, value)?.trim_start_matches("\\\\").into(),
        "ROLE" => options.role = require_value(name, value)?.trim_start_matches("\\\\").into(),
        "NODE" => {
            options.nodes = require_value(name, value)?
                .split(',')
                .map(|s| s.trim_matches('"').to_owned())
                .filter(|s| !s.is_empty())
                .collect();
            if options.nodes.is_empty() {
                return Err(ParseError("/NODE requires at least one computer".into()));
            }
        }
        "IMPLEVEL" => options.impersonation = require_value(name, value)?.into(),
        "AUTHLEVEL" => options.authentication = require_value(name, value)?.into(),
        "LOCALE" => options.locale = require_value(name, value)?.into(),
        "PRIVILEGES" => options.privileges = on_off(name, value)?,
        "TRACE" => options.trace = on_off(name, value)?,
        "RECORD" => options.record = Some(require_value(name, value)?.into()),
        "INTERACTIVE" => options.interactive = on_off(name, value)?,
        "FAILFAST" => {
            let value = require_value(name, value)?;
            options.fail_fast = match value.to_ascii_lowercase().as_str() {
                "on" => FailFast::On,
                "off" => FailFast::Off,
                _ => FailFast::Timeout(value.parse().map_err(|_| ParseError(format!("Invalid /FAILFAST value: {value}")))?),
            };
        }
        "USER" => options.user = Some(require_value(name, value)?.into()),
        "PASSWORD" => options.password = Some(require_value(name, value)?.into()),
        "AUTHORITY" => options.authority = Some(require_value(name, value)?.into()),
        "AGGREGATE" => options.aggregate = on_off(name, value)?,
        "OUTPUT" | "APPEND" => {
            let append = name.eq_ignore_ascii_case("APPEND");
            let target = require_value(name, value)?;
            options.destination = if target.eq_ignore_ascii_case("stdout") {
                Destination::Stdout
            } else if target.eq_ignore_ascii_case("clipboard") {
                Destination::Clipboard { append }
            } else {
                Destination::File { path: target.into(), append }
            };
        }
        _ => return Err(ParseError(format!("Invalid global switch: {token}"))),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(args: &[&str]) -> Invocation {
        parse(args.iter().copied()).unwrap()
    }

    #[test]
    fn empty_and_help_are_global_help() {
        assert_eq!(command(&[]).command, Command::Help(HelpTopic::Global));
        assert_eq!(command(&["/?"]).command, Command::Help(HelpTopic::Global));
        assert_eq!(command(&["/?:FULL"]).command, Command::Help(HelpTopic::Global));
    }

    #[test]
    fn global_switch_help_does_not_require_a_value() {
        assert_eq!(
            command(&["/node", "/?"]).command,
            Command::Help(HelpTopic::Switch("NODE".into()))
        );
    }

    #[test]
    fn parses_case_insensitive_alias_get_and_switches() {
        let invocation = command(&[
            "/namespace:root\\cimv2",
            "PrOcEsS",
            "where",
            "Name='cmd.exe'",
            "GET",
            "Name,ProcessId",
            "/value",
        ]);
        let Command::Query(query) = invocation.command else { panic!() };
        assert_eq!(query.source.class, "Win32_Process");
        assert_eq!(query.filter.as_deref(), Some("Name='cmd.exe'"));
        assert_eq!(query.verb, Verb::Get(vec!["Name".into(), "ProcessId".into()]));
        assert_eq!(query.format, OutputFormat::Value);
    }

    #[test]
    fn path_escapes_alias_mode() {
        let Command::Query(query) = command(&["path", "Win32_OperatingSystem", "get", "Caption"]).command else { panic!() };
        assert_eq!(query.source.kind, SourceKind::Path);
        assert_eq!(query.source.class, "Win32_OperatingSystem");
    }

    #[test]
    fn parses_all_global_switch_shapes() {
        let invocation = command(&[
            "/node:server-a,server-b", "/implevel:delegate", "/authlevel:pktPrivacy",
            "/locale:ms_411", "/privileges:enable", "/trace:on", "/record:trace.xml",
            "/interactive:off", "/failfast:1250", "/user:DOMAIN\\user", "/password:secret",
            "/aggregate:off", "/authority:ntlmdomain:DOMAIN", "/append:out.txt", "context",
        ]);
        assert_eq!(invocation.globals.nodes, ["server-a", "server-b"]);
        assert_eq!(invocation.globals.fail_fast, FailFast::Timeout(1250));
        assert_eq!(invocation.globals.destination, Destination::File { path: "out.txt".into(), append: true });
        assert!(!invocation.globals.interactive);
        assert!(!invocation.globals.aggregate);
    }

    #[test]
    fn parses_assoc_switches() {
        let Command::Query(query) = command(&[
            "os", "assoc", "/resultclass:Win32_ComputerSystem", "/resultrole:GroupComponent",
            "/assocclass:Win32_SystemOperatingSystem",
        ]).command else { panic!() };
        let Verb::Assoc(options) = query.verb else { panic!() };
        assert_eq!(options.result_class.as_deref(), Some("Win32_ComputerSystem"));
        assert_eq!(options.result_role.as_deref(), Some("GroupComponent"));
        assert_eq!(options.assoc_class.as_deref(), Some("Win32_SystemOperatingSystem"));
    }

    #[test]
    fn parses_mutating_verbs_without_executing_them() {
        assert!(matches!(command(&["process", "where", "ProcessId=1", "delete"]).command, Command::Query(query) if matches!(query.verb, Verb::Delete)));
        assert!(matches!(command(&["service", "where", "Name='x'", "call", "StartService"]).command, Command::Query(query) if matches!(query.verb, Verb::Call { .. })));
        assert!(matches!(command(&["environment", "create", "Name='x',VariableValue='y'"]).command, Command::Query(query) if matches!(query.verb, Verb::Create(_))));
    }

    #[test]
    fn invalid_inputs_have_stable_diagnostics() {
        assert_eq!(parse(["missingalias"]).unwrap_err().0, "Alias not found: missingalias");
        assert_eq!(parse(["process", "get"]).unwrap_err().0, "GET requires at least one property");
        assert_eq!(parse(["process", "where"]).unwrap_err().0, "WHERE requires a condition");
        assert_eq!(parse(["/failfast:nope", "context"]).unwrap_err().0, "Invalid /FAILFAST value: nope");
    }
}
