use crate::cli::OutputFormat;
use crate::engine::{Record, Value};

pub fn render(records: &[Record], requested: &[String], format: &OutputFormat, node: &str) -> String {
    let columns = columns(records, requested);
    match format {
        OutputFormat::Value | OutputFormat::List => value(records, &columns),
        OutputFormat::Csv => csv(records, &columns, node),
        OutputFormat::RawXml => raw_xml(records, &columns),
        OutputFormat::HForm => html(records, &columns, false),
        OutputFormat::HTable => html(records, &columns, true),
        OutputFormat::Table | OutputFormat::Custom(_) => table(records, &columns),
    }
}

fn columns(records: &[Record], requested: &[String]) -> Vec<String> {
    if !requested.is_empty() && requested.iter().any(|p| p != "*") {
        return requested
            .iter()
            .map(|wanted| {
                records
                    .iter()
                    .flat_map(|record| record.keys())
                    .find(|actual| actual.eq_ignore_ascii_case(wanted))
                    .cloned()
                    .unwrap_or_else(|| wanted.clone())
            })
            .collect();
    }
    records.first().map_or_else(Vec::new, |record| record.keys().cloned().collect())
}

fn cell<'a>(record: &'a Record, column: &str) -> Option<&'a Value> {
    record
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(column))
        .map(|(_, value)| value)
}

fn table(records: &[Record], columns: &[String]) -> String {
    if records.is_empty() || columns.is_empty() {
        return "No Instance(s) Available.\r\r\n".into();
    }
    let widths: Vec<usize> = columns
        .iter()
        .map(|column| {
            records
                .iter()
                .filter_map(|record| cell(record, column))
                .map(|value| value.to_string().chars().count())
                .chain([column.chars().count()])
                .max()
                .unwrap_or(0)
                + 2
        })
        .collect();
    let mut out = String::new();
    write_row(&mut out, columns.iter().map(String::as_str), &widths);
    for record in records {
        let values: Vec<String> = columns
            .iter()
            .map(|column| cell(record, column).map(ToString::to_string).unwrap_or_default())
            .collect();
        write_row(&mut out, values.iter().map(String::as_str), &widths);
    }
    out
}

fn write_row<'a>(out: &mut String, values: impl Iterator<Item = &'a str>, widths: &[usize]) {
    for (value, width) in values.zip(widths) {
        out.push_str(value);
        out.extend(std::iter::repeat_n(' ', width.saturating_sub(value.chars().count())));
    }
    out.push_str("\r\r\n");
}

fn value(records: &[Record], columns: &[String]) -> String {
    if records.is_empty() {
        return "No Instance(s) Available.\r\r\n".into();
    }
    let mut out = String::new();
    for record in records {
        out.push_str("\r\r\n");
        for column in columns {
            let value = cell(record, column).map(ToString::to_string).unwrap_or_default();
            out.push_str(column);
            out.push('=');
            out.push_str(&value);
            out.push_str("\r\r\n");
        }
    }
    out.push_str("\r\r\n");
    out
}

fn csv(records: &[Record], columns: &[String], node: &str) -> String {
    if records.is_empty() {
        return "No Instance(s) Available.\r\r\n".into();
    }
    let mut out = String::from("\r\r\nNode");
    for column in columns {
        out.push(',');
        out.push_str(&csv_escape(column));
    }
    out.push_str("\r\r\n");
    for record in records {
        out.push_str(&csv_escape(node));
        for column in columns {
            out.push(',');
            out.push_str(&csv_escape(
                &cell(record, column).map(ToString::to_string).unwrap_or_default(),
            ));
        }
        out.push_str("\r\r\n");
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.chars().any(|character| matches!(character, ',' | '"' | '\r' | '\n')) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.into()
    }
}

fn raw_xml(records: &[Record], columns: &[String]) -> String {
    let mut out = String::from("<?xml version=\"1.0\"?>\r\n<COMMAND>\r\n<RESULTS>\r\n");
    for record in records {
        out.push_str("<CIM>\r\n<INSTANCE>\r\n");
        for column in columns {
            let value = cell(record, column).map(ToString::to_string).unwrap_or_default();
            out.push_str(&format!(
                "<PROPERTY NAME=\"{}\"><VALUE>{}</VALUE></PROPERTY>\r\n",
                xml_escape(column),
                xml_escape(&value)
            ));
        }
        out.push_str("</INSTANCE>\r\n</CIM>\r\n");
    }
    out.push_str("</RESULTS>\r\n</COMMAND>\r\n");
    out
}

fn html(records: &[Record], columns: &[String], as_table: bool) -> String {
    let mut out = String::from("<HTML><HEAD><META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=utf-8\"></HEAD><BODY>");
    if as_table {
        out.push_str("<TABLE BORDER=\"1\"><TR>");
        for column in columns {
            out.push_str(&format!("<TH>{}</TH>", xml_escape(column)));
        }
        out.push_str("</TR>");
        for record in records {
            out.push_str("<TR>");
            for column in columns {
                let value = cell(record, column).map(ToString::to_string).unwrap_or_default();
                out.push_str(&format!("<TD>{}</TD>", xml_escape(&value)));
            }
            out.push_str("</TR>");
        }
        out.push_str("</TABLE>");
    } else {
        for record in records {
            for column in columns {
                let value = cell(record, column).map(ToString::to_string).unwrap_or_default();
                out.push_str(&format!("<B>{}</B>={}<BR>", xml_escape(column), xml_escape(&value)));
            }
            out.push_str("<BR>");
        }
    }
    out.push_str("</BODY></HTML>\r\n");
    out
}

fn xml_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn fixtures() -> Vec<Record> {
        [
            [("Name", Value::String("alpha".into())), ("Pid", Value::Unsigned(7))],
            [("Name", Value::String("a,b\"c".into())), ("Pid", Value::Null)],
        ]
        .into_iter()
        .map(|pairs| pairs.into_iter().map(|(k, v)| (k.into(), v)).collect::<BTreeMap<_, _>>())
        .collect()
    }

    #[test]
    fn table_padding_and_wmic_line_endings_are_stable() {
        assert_eq!(
            render(&fixtures()[..1], &["Name".into(), "Pid".into()], &OutputFormat::Table, "NODE"),
            "Name   Pid  \r\r\nalpha  7    \r\r\n"
        );
    }

    #[test]
    fn value_format_has_object_separators_and_empty_null() {
        assert_eq!(
            render(&fixtures()[1..], &["Name".into(), "Pid".into()], &OutputFormat::Value, "NODE"),
            "\r\r\nName=a,b\"c\r\r\nPid=\r\r\n\r\r\n"
        );
    }

    #[test]
    fn csv_quotes_only_when_required() {
        assert_eq!(
            render(&fixtures(), &["Name".into()], &OutputFormat::Csv, "BOX"),
            "\r\r\nNode,Name\r\r\nBOX,alpha\r\r\nBOX,\"a,b\"\"c\"\r\r\n"
        );
    }

    #[test]
    fn arrays_use_wmic_brace_separator_syntax() {
        assert_eq!(Value::Array(vec![Value::String("a".into()), Value::Unsigned(2)]).to_string(), "{a; 2}");
    }

    #[test]
    fn empty_results_are_explicit() {
        assert_eq!(render(&[], &[], &OutputFormat::Table, "BOX"), "No Instance(s) Available.\r\r\n");
    }
}
