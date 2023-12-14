
use anyhow::{Context, Error};
use clap::Parser;
use isomdl::presentation::{device::Document, Stringify};
use serde_cbor::Value as CborValue;
mod args;
use args::{IsoMdlArgs, IsoMdlCommand, IssueCommand, VerifyCommand};
use serde_json::Value as JsonValue;
use std::{
    path::Path,
    fs::File,
    io::{BufWriter, Write}
};

fn main() -> Result<(), Error> {
    let args : IsoMdlArgs = IsoMdlArgs::parse();

    match args.subcommand {
        IsoMdlCommand::Issue(issue_command) => {
            issue_mdl(issue_command)
        },
        IsoMdlCommand::Verify(verify_command) => {
            verify_mdl(verify_command)
        }
    }
}

fn issue_mdl( issue_command: IssueCommand ) -> Result<(), Error> {
    let content = std::fs::read_to_string(&issue_command.input_filename)
        .context(format!("could not read input_filename {}", &issue_command.input_filename))?;

    let parsed_json: JsonValue = serde_json::from_str(&content)?;

    let isomdl_data: &JsonValue = &parsed_json["namespace"]["org.iso.18013.5.1"];
    let aamva_isomdl_data: &JsonValue = &parsed_json["namespace"]["org.iso.18013.5.1.aamva"];

    if Some(isomdl_data).is_some() && Some(aamva_isomdl_data).is_some() {
        let out_writer = match issue_command.output_filename {
            Some(x) => {
                let path = Path::new(&x);
                if path.exists() {
                    eprintln!("EEXIST=17 output_filename  \"{}\" already exists", path.to_string_lossy());
                    std::process::exit(17);
                }
                Box::new(File::create(&path).unwrap()) as Box<dyn Write>
            }
            None => Box::new(std::io::stdout()) as Box<dyn Write>,
        };
        let buf = BufWriter::new(out_writer);
        let _ = isomdl::issuance::mdoc::aq_issue::aq_issue(&isomdl_data, &aamva_isomdl_data, buf);
    }

    Ok(())
}


fn verify_mdl( verify_command: VerifyCommand ) -> Result<(), Error> {
    let content = std::fs::read_to_string(&verify_command.input_filename)
        .context(format!("could not read input_filename {}", &verify_command.input_filename))?;

    let out_writer = match verify_command.output_filename {
        Some(x) => {
            let path = Path::new(&x);
            if path.exists() {
                eprintln!("EEXIST=17 output_filename \"{}\" already exists", path.to_string_lossy());
                std::process::exit(17);
            }
        Box::new(File::create(&path).unwrap()) as Box<dyn Write>
        }
        None => Box::new(std::io::stdout()) as Box<dyn Write>,
    };
    let buf = BufWriter::new(out_writer);
    print_claims(content, buf)
}

fn print_claims(mdl: String, mut output_buffer: BufWriter<Box<dyn Write>>) -> Result<(), Error> {

    let parsed = Document::parse(mdl).context("could not parse mdl");
    let namespaces = parsed?.namespaces;
    for namespace in namespaces.into_inner() {
        let namespace_name = namespace.0;
        writeln!(output_buffer, "namespace={:#?}", namespace_name)
            .context("Error writing to output file")?;
        for key in namespace.1.into_inner() {
            let key_name = key.0;
            let value = key.1.into_inner().element_value;
            
            // println!("{}", serde_json::to_string_pretty(&key.1.into_inner())?);
            let string_value = convert_value_to_string(value);
            if key_name != "portrait" {
                writeln!(output_buffer, "  '{}': {}", key_name, string_value)
                    .context("Error writing to output file")?;
            }
        }
    }
    
    Ok(())
}
fn convert_value_to_string(value: CborValue) -> String {
    let string_value;
    match value {
        CborValue::Null => string_value=String::from(""),
        CborValue::Text (v) => string_value=format!("'{}'", serde_cbor::value::from_value::<String>(serde_cbor::Value::Text(v)).unwrap()),
        CborValue::Bool (v) => string_value=format!("bool={}", serde_cbor::value::from_value::<bool>(serde_cbor::Value::Bool(v)).unwrap().to_string()),
        CborValue::Integer (v) => string_value=format!("int={}", serde_cbor::value::from_value::<u128>(serde_cbor::Value::Integer(v)).unwrap().to_string()),
        CborValue::Float (v) => string_value=format!("int={}", serde_cbor::value::from_value::<f64>(serde_cbor::Value::Float(v)).unwrap().to_string()),
        CborValue::Bytes (_v) => string_value=String::from("bytes"),
        CborValue::Array (_v) => {
            let mut s = String::from("[ ").to_owned();
            for val in _v {
                s.push_str(&convert_value_to_string(val));
                s.push_str(", ");
            }
            s.push_str("]");
            string_value = s;
        },
        CborValue::Map (_v) => {
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
        CborValue::Tag (_a, _v) => string_value=format!("tag=({}, {})", convert_tag_to_string(_a), convert_value_to_string(*_v)),
        CborValue::__Hidden => string_value=String::from(""),
    }
    return string_value;
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


