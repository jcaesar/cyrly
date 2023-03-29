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
    // Tagged(Box<(String, Value)>),
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
            // Value::Tagged(bx) => Tagged(Box::new(serde_yaml::value::TaggedValue {
            //     tag: serde_yaml::value::Tag::new(bx.0),
            //     value: bx.1.into(),
            // })),
        }
    }
}

fuzz_target!(|data: Value| {
    let data: serde_yaml::Value = data.into();

    let mut out = String::new();

    data.serialize(curly_yaml::CurlySerializer::new(&mut out))
        .unwrap();

    let de = serde_yaml::from_str::<serde_yaml::Value>(&out);
    if !de.as_ref().map_or(false, |de| de == &data) {
        println!(
            "---\n# Serialized with serde_yaml\n{}\n\n",
            serde_yaml::to_string(&data).unwrap()
        );
        println!("---\n# Serialized with curly_yaml\n{out}");
        dbg!(data, de.unwrap());
        panic!()
    }
});
