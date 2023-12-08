use std::collections::BTreeMap;

use anyhow::{Context, Error};
use clap::Parser;
use clap_stdin::MaybeStdin;
use isomdl::presentation::{device::Document, Stringify};
use serde_cbor::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, clap::Subcommand)]
enum Action {
    /// Print the namespaces and element identifiers used in an mDL.
    GetNamespaces {
        /// Base64 encoded mDL in the format used in the issuance module of this crate.
        mdl: MaybeStdin<String>,
    },
    /// Print the namespaces and element identifiers used in an mDL.
    GetClaims {
        /// Base64 encoded mDL in the format used in the issuance module of this crate.
        mdl: MaybeStdin<String>,
    },
    // create a new mdoc based on a JSON string passed in on the command line
    // CreateMdoc {
    //     /// Base64 encoded mDL in the format used in the issuance module of this crate.
    //     json: MaybeStdin<String>,
    // },
}

fn main() -> Result<(), Error> {
    match Args::parse().action {
        Action::GetNamespaces { mdl } => {
            print_namespaces(mdl.to_string())
        },
        Action::GetClaims { mdl } => {
            print_claims(mdl.to_string())
        },
        // Action::CreateMdoc { json } => {
        //     create_mdoc_from_json_string( json.to_string())
        // },
    }
}

fn print_namespaces(mdl: String) -> Result<(), Error> {
    let claims = Document::parse(mdl)
        .context("could not parse mdl")?
        .namespaces
        .into_inner()
        .into_iter()
        .map(|(ns, inner)| (ns, inner.into_inner().into_keys().collect()))
        .collect::<BTreeMap<String, Vec<String>>>();
    println!("{}", serde_json::to_string_pretty(&claims)?);
    Ok(())
}

fn convert_tag_to_string(value: u64) -> String {
    if value == 0 {
        return String::from("IsoDateTime(Z)");
    }
    if value == 1004 {
        return String::from("DateStr");
    }
    return format!("{}", value);
}

fn convert_value_to_string(value: serde_cbor::value::Value) -> String {
    let string_value;
    match value {
        Value::Null => string_value=String::from(""),
        Value::Text (v) => string_value=format!("'{}'", serde_cbor::value::from_value::<String>(serde_cbor::Value::Text(v)).unwrap()),
        Value::Bool (v) => string_value=format!("bool={}", serde_cbor::value::from_value::<bool>(serde_cbor::Value::Bool(v)).unwrap().to_string()),
        Value::Integer (v) => string_value=format!("int={}", serde_cbor::value::from_value::<u128>(serde_cbor::Value::Integer(v)).unwrap().to_string()),
        Value::Float (v) => string_value=format!("int={}", serde_cbor::value::from_value::<f64>(serde_cbor::Value::Float(v)).unwrap().to_string()),
        Value::Bytes (_v) => string_value=String::from("bytes"),
        Value::Array (_v) => {
            let mut s = String::from("[ ").to_owned();
            for val in _v {
                s.push_str(&convert_value_to_string(val));
                s.push_str(", ");
            }
            s.push_str("]");
            string_value = s;
        },
        Value::Map (_v) => {
            let mut s: String = String::from("{ ");
            for (key, val) in _v {
                s.push_str(&convert_value_to_string(key));
                s.push_str(": ");
                s.push_str(&convert_value_to_string(val));
            }
            s.push_str("}");
            string_value = s;
        },
        //string_value=String::from("{}"),//string_value=String::from(format!("{}", convert_value_to_string(serde_cbor::Value::Map(_v)))),
        Value::Tag (_a, _v) => string_value=format!("tag=({}, {})", convert_tag_to_string(_a), convert_value_to_string(*_v)),
        Value::__Hidden => string_value=String::from(""),
    }
    return string_value;
}

fn print_claims(mdl: String) -> Result<(), Error> {

    let parsed = Document::parse(mdl).context("could not parse mdl");
    let namespaces = parsed?.namespaces;
    for namespace in namespaces.into_inner() {
        let namespace_name = namespace.0;
        println!("namespace={:#?}", namespace_name);
        for key in namespace.1.into_inner() {
            let key_name = key.0;
            let value = key.1.into_inner().element_value;
            
            // println!("{}", serde_json::to_string_pretty(&key.1.into_inner())?);
            let string_value = convert_value_to_string(value);
            if key_name != "portrait" {
                println!("  '{}': {}", key_name, string_value);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn print_namespaces() {
        super::print_namespaces(include_str!("../test/stringified-mdl.txt").to_string()).unwrap()
    }
}
