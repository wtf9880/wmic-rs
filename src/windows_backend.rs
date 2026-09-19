use crate::engine::{Backend, Record, Value};
use std::collections::HashMap;
use wmi::{Variant, WMIConnection};

pub struct WindowsWmi;

impl Backend for WindowsWmi {
    fn query(&self, namespace: &str, node: &str, wql: &str) -> Result<Vec<Record>, String> {
        let namespace_path = if node.eq_ignore_ascii_case("localhost") || node == "." {
            namespace.to_owned()
        } else {
            format!(r"\\{node}\{namespace}")
        };
        let connection = WMIConnection::with_namespace_path(&namespace_path)
            .map_err(|error| error.to_string())?;
        let rows: Vec<HashMap<String, Variant>> =
            connection.raw_query(wql).map_err(|error| error.to_string())?;
        Ok(rows
            .into_iter()
            .map(|row| row.into_iter().map(|(key, value)| (key, convert(value))).collect())
            .collect())
    }
}

fn convert(value: Variant) -> Value {
    match value {
        Variant::Empty | Variant::Null => Value::Null,
        Variant::String(value) => Value::String(value),
        Variant::I1(value) => Value::Signed(value.into()),
        Variant::I2(value) => Value::Signed(value.into()),
        Variant::I4(value) => Value::Signed(value.into()),
        Variant::I8(value) => Value::Signed(value),
        Variant::UI1(value) => Value::Unsigned(value.into()),
        Variant::UI2(value) => Value::Unsigned(value.into()),
        Variant::UI4(value) => Value::Unsigned(value.into()),
        Variant::UI8(value) => Value::Unsigned(value),
        Variant::R4(value) => Value::Float(value.into()),
        Variant::R8(value) => Value::Float(value),
        Variant::Bool(value) => Value::Bool(value),
        Variant::Array(values) => Value::Array(values.into_iter().map(convert).collect()),
        Variant::Unknown(value) => Value::Object(format!("{value:?}")),
        Variant::Object(value) => Value::Object(format!("{value:?}")),
    }
}
