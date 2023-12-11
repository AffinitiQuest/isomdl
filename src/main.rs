
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

use isomdl::definitions::traits::{FromJson, ToNamespaceMap};
use isomdl::definitions::namespaces::org_iso_18013_5_1::OrgIso1801351;

fn main() -> Result<(), Error> {
    let args : IsoMdlArgs = IsoMdlArgs::parse();
    println!("{:?}", args);

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

    let v: JsonValue = serde_json::from_str(&content)?;
    // iterate over the namespaces
    let namespaces: &JsonValue = &v["namespace"];
    match namespaces {
        JsonValue::Object(o) => {
            for namespace in o.keys() {
                let value = o.get_key_value(namespace);
                println!("namespace={}", namespace);
                let doc_type = String::from("org.iso.18013.5.1.mDL");
                
                let mdl_data = OrgIso1801351::from_json (value)         
                    .unwrap()
                    .to_ns_map();            
            }
        },
        __hidden => {}
    }

    Ok(())
    // let out_writer = match issue_command.output_filename {
    //     Some(x) => {
    //         let path = Path::new(&x);
    //         Box::new(File::create(&path).unwrap()) as Box<dyn Write>
    //     }
    //     None => Box::new(std::io::stdout()) as Box<dyn Write>,
    // };

    // let mut buf = BufWriter::new(out_writer);
    // writeln!(buf, "{:?}", &content)
    //     .context("Error writing to output file")
}

// fn minimal_mdoc_builder() -> Builder {
//     let doc_type = String::from("org.iso.18013.5.1.mDL");
//     let isomdl_namespace = String::from("org.iso.18013.5.1");
//     let aamva_namespace = String::from("org.iso.18013.5.1.aamva");

//     let isomdl_data = OrgIso1801351::from_json(&isomdl_data())
//         .unwrap()
//         .to_ns_map();
//     let aamva_data = OrgIso1801351Aamva::from_json(&aamva_isomdl_data())
//         .unwrap()
//         .to_ns_map();

//     let namespaces = [
//         (isomdl_namespace, isomdl_data),
//         (aamva_namespace, aamva_data),
//     ]
//     .into_iter()
//     .collect();

//     let validity_info = ValidityInfo {
//         signed: OffsetDateTime::now_utc(),
//         valid_from: OffsetDateTime::now_utc(),
//         valid_until: OffsetDateTime::now_utc(),
//         expected_update: None,
//     };

//     let digest_algorithm = DigestAlgorithm::SHA256;

//     let der = include_str!("../../test/issuance/device_key.b64");
//     let der_bytes = base64::decode(der).unwrap();
//     let key = p256::SecretKey::from_sec1_der(&der_bytes).unwrap();
//     let pub_key = key.public_key();
//     let ec = pub_key.to_encoded_point(false);
//     let x = ec.x().unwrap().to_vec();
//     let y = EC2Y::Value(ec.y().unwrap().to_vec());
//     let device_key = CoseKey::EC2 {
//         crv: EC2Curve::P256,
//         x,
//         y,
//     };

//     let device_key_info = DeviceKeyInfo {
//         device_key,
//         key_authorizations: None,
//         key_info: None,
//     };

//     Mdoc::builder()
//         .doc_type(doc_type)
//         .namespaces(namespaces)
//         .validity_info(validity_info)
//         .digest_algorithm(digest_algorithm)
//         .device_key_info(device_key_info)
// }


fn verify_mdl( verify_command: VerifyCommand ) -> Result<(), Error> {
    let content = std::fs::read_to_string(&verify_command.input_filename)
        .context(format!("could not read input_filename {}", &verify_command.input_filename))?;

    let out_writer = match verify_command.output_filename {
        Some(x) => {
            let path = Path::new(&x);
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


