//! Reads the Go package at the repository root and compares it with this
//! crate.
//!
//! Risk: a message type, payload, field or closed value is added on one side
//! only. The other side would drop it without an error, and the hub and an
//! agent would stop agreeing while both test suites stay green.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use labtether_protocol as protocol;
use serde_json::{Map, Value, json};

struct GoField {
    json_name: String,
    go_type: String,
    omit_empty: bool,
}

#[derive(Default)]
struct GoPackage {
    structs: BTreeMap<String, Vec<GoField>>,
    aliases: BTreeMap<String, String>,
    closed_strings: BTreeMap<String, BTreeSet<String>>,
    message_types: BTreeSet<String>,
}

fn unquote(token: &str) -> Option<&str> {
    token.strip_prefix('"')?.strip_suffix('"')
}

fn parse_field(line: &str) -> GoField {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let tag = tokens
        .get(2)
        .and_then(|token| token.strip_prefix("`json:\"")?.strip_suffix("\"`"))
        .unwrap_or_else(|| panic!("Go field without a json tag: {line}"));
    let mut options = tag.split(',');
    GoField {
        json_name: options.next().unwrap_or_default().to_owned(),
        go_type: tokens[1].to_owned(),
        omit_empty: options.any(|option| option == "omitempty"),
    }
}

fn read_go_package() -> GoPackage {
    let root = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    let mut package = GoPackage::default();
    let mut typed_constants = Vec::new();

    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".go") || name.ends_with("_test.go") {
            continue;
        }
        let mut open_struct: Option<String> = None;
        for line in std::fs::read_to_string(&path).unwrap().lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if let Some(current) = &open_struct {
                if line == "}" {
                    open_struct = None;
                } else if !tokens.is_empty() && !tokens[0].starts_with("//") {
                    package
                        .structs
                        .get_mut(current)
                        .unwrap()
                        .push(parse_field(line));
                }
                continue;
            }
            match tokens.as_slice() {
                ["type", name, "struct", "{"] => {
                    package.structs.insert((*name).to_owned(), Vec::new());
                    open_struct = Some((*name).to_owned());
                }
                ["type", name, "=", target] => {
                    package
                        .aliases
                        .insert((*name).to_owned(), (*target).to_owned());
                }
                ["type", name, "string"] => {
                    package
                        .closed_strings
                        .entry((*name).to_owned())
                        .or_default();
                }
                [name, "=", value, ..] if name.starts_with("Msg") => {
                    package
                        .message_types
                        .insert(unquote(value).unwrap().to_owned());
                }
                [_, type_name, "=", value] => {
                    if let Some(value) = unquote(value) {
                        typed_constants.push(((*type_name).to_owned(), value.to_owned()));
                    }
                }
                _ => {}
            }
        }
    }
    for (type_name, value) in typed_constants {
        if let Some(values) = package.closed_strings.get_mut(&type_name) {
            values.insert(value);
        }
    }
    package
}

#[derive(Clone, Copy, PartialEq)]
enum Fill {
    /// Every field set, every list and map holding one item.
    Full,
    /// As `Full`, but every list and map is `null`, as Go writes a nil one.
    NullCollections,
}

fn sample(package: &GoPackage, go_type: &str, fill: Fill) -> Value {
    if let Some(inner) = go_type.strip_prefix('*') {
        return sample(package, inner, fill);
    }
    let item = go_type.strip_prefix("[]").map(|inner| (inner, false));
    let item = item.or(go_type
        .strip_prefix("map[string]")
        .map(|inner| (inner, true)));
    if let Some((inner, is_map)) = item {
        let value = sample(package, inner, fill);
        return match (fill, is_map) {
            (Fill::NullCollections, _) => Value::Null,
            (Fill::Full, true) => json!({ "k": value }),
            (Fill::Full, false) => json!([value]),
        };
    }
    match go_type {
        "string" => json!("s"),
        "bool" => json!(true),
        "int" | "int64" | "uint64" => json!(7),
        "float64" => json!(1.5),
        name if package.closed_strings.contains_key(name) => json!("s"),
        name => {
            let fields = package
                .structs
                .get(name)
                .unwrap_or_else(|| panic!("Go type {name} is not handled by this test"));
            let object = fields.iter().map(|field| {
                (
                    field.json_name.clone(),
                    sample(package, &field.go_type, fill),
                )
            });
            Value::Object(object.collect())
        }
    }
}

/// What a default Rust value must encode to: Go's zero value, with `[]` and
/// `{}` where Go writes `null` for a nil list or map.
fn zero(package: &GoPackage, go_type: &str) -> Value {
    if go_type.starts_with('*') {
        return Value::Null;
    }
    if go_type.starts_with("[]") {
        return json!([]);
    }
    if go_type.starts_with("map[") {
        return json!({});
    }
    match go_type {
        "string" => json!(""),
        "bool" => json!(false),
        "int" | "int64" | "uint64" => json!(0),
        "float64" => json!(0.0),
        name if package.closed_strings.contains_key(name) => json!(""),
        name => {
            let mut object = Map::new();
            for field in package.structs[name]
                .iter()
                .filter(|field| !field.omit_empty)
            {
                object.insert(field.json_name.clone(), zero(package, &field.go_type));
            }
            Value::Object(object)
        }
    }
}

fn names<V>(map: &BTreeMap<impl AsRef<str>, V>) -> BTreeSet<String> {
    map.keys().map(|key| key.as_ref().to_owned()).collect()
}

#[test]
fn payloads_have_the_same_fields_as_go() {
    let mut package = read_go_package();
    let envelope = package
        .structs
        .remove("Message")
        .expect("Go Message envelope");
    let envelope: Vec<&str> = envelope
        .iter()
        .map(|field| field.json_name.as_str())
        .collect();
    assert_eq!(
        envelope,
        ["type", "id", "data"],
        "the Go envelope changed; update Message"
    );

    let payloads = common::payloads();
    assert_eq!(
        names(&package.structs),
        names(&payloads),
        "payload types differ from Go"
    );

    for (name, payload) in &payloads {
        let full = sample(&package, name, Fill::Full);
        let round_trip =
            (payload.round_trip)(full.clone()).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(round_trip, full, "{name}: a field differs from Go");

        if let Err(error) = (payload.round_trip)(sample(&package, name, Fill::NullCollections)) {
            panic!("{name}: rejects the null Go writes for a nil list or map: {error}");
        }

        assert_eq!(
            (payload.zero)(),
            zero(&package, name),
            "{name}: omitempty differs from Go"
        );
    }
}

#[test]
fn message_types_and_closed_values_match_go() {
    let package = read_go_package();

    let known: BTreeSet<String> = protocol::KNOWN_MESSAGE_TYPES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    assert_eq!(
        known.len(),
        protocol::KNOWN_MESSAGE_TYPES.len(),
        "duplicate message type"
    );
    assert_eq!(package.message_types, known, "message types differ from Go");

    let closed = common::closed_strings();
    assert_eq!(
        names(&package.closed_strings),
        names(&closed),
        "closed string types differ"
    );
    for (name, values) in &closed {
        let values: BTreeSet<String> = values.iter().map(|value| (*value).to_owned()).collect();
        assert_eq!(
            package.closed_strings[*name], values,
            "{name}: values differ from Go"
        );
    }

    // A type alias has no fields of its own, so it is checked by name.
    let aliases = BTreeMap::from([(
        "SSHKeyRemoveData".to_owned(),
        "SSHKeyInstallData".to_owned(),
    )]);
    assert_eq!(package.aliases, aliases, "Go type aliases changed");
    let _: fn() -> protocol::SSHKeyRemoveData = protocol::SSHKeyInstallData::default;
}
