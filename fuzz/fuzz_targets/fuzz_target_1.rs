#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::Serialize;

#[derive(Arbitrary, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Sequence(Vec<Value>),
    Mapping(Vec<(Value, Value)>),
    // TaggdValues are serialized by recognizing 1-element maps with a "!…" key in serde_yaml.
    // I don't think I want to reproduce that hackery in my little lib
}

impl Into<serde_yaml::Value> for Value {
    fn into(self) -> serde_yaml::Value {
        use serde_yaml::Value::*;
        match self {
            Value::Null => Null,
            Value::Bool(b) => Bool(b),
            Value::Number(f) => Number(f.into()),
            Value::String(s) => String(s),
            Value::Sequence(s) => Sequence(s.into_iter().map(Into::into).collect()),
            Value::Mapping(s) => {
                Mapping(s.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
            }
        }
    }
}

#[derive(Arbitrary, Debug)]
struct Problem {
    multiline: bool,
    data: Value,
}

fuzz_target!(|problem: Problem| {
    let data: serde_yaml::Value = problem.data.into();

    let mut out = String::new();

    let mut curl = curly_yaml::CurlySerializer::new(&mut out);
    curl.multiline = problem.multiline;
    data.serialize(curl).unwrap();

    let de = serde_yaml::from_str::<serde_yaml::Value>(&out);
    if cfg!(feature = "debuglog") {
        let yamlser = serde_yaml::to_string(&data);
        match &yamlser {
            Ok(yamlser) => println!("---\n# Serialized with serde_yaml\n{}\n# END", yamlser),
            Err(_) => {
                dbg!(&yamlser);
            }
        }
        println!("---\n# Serialized with curly_yaml\n{out}\n# END");
        dbg!(&data, problem.multiline, de.as_ref().unwrap());
        dbg!(serde_yaml::from_str::<serde_yaml::Value>(&yamlser.unwrap())).ok();
    }
    if !de.as_ref().map_or(false, |de| de == &data) {
        panic!()
    }
});
