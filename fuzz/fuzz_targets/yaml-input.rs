#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::ser::Serialize;
use serde_yaml::Value;

fn has_tagged_vals_or_floats(val: &Value) -> bool {
    match val {
        serde_yaml::Value::Null => false,
        serde_yaml::Value::Bool(_) => false,
        serde_yaml::Value::Number(n) => !n.is_i64(),
        serde_yaml::Value::String(_) => false,
        serde_yaml::Value::Sequence(s) => s.iter().any(has_tagged_vals_or_floats),
        serde_yaml::Value::Mapping(m) => m
            .iter()
            .any(|(k, v)| has_tagged_vals_or_floats(k) || has_tagged_vals_or_floats(v)),
        serde_yaml::Value::Tagged(_) => true,
    }
}

fuzz_target!(|data: &str| {
    let val = serde_yaml::from_str::<Value>(data);
    let val = match val {
        Ok(val) => val,
        Err(_) => return,
    };
    if has_tagged_vals_or_floats(&val) {
        return;
    }

    let mut out = String::new();

    val.serialize(curly_yaml::CurlySerializer::new(&mut out))
        .unwrap();
    let de = serde_yaml::from_str::<serde_yaml::Value>(&out);

    if cfg!(feature = "print") {
        println!(
            "---\n# Serialized with serde_yaml\n{}\n\n",
            serde_yaml::to_string(&data).unwrap()
        );
        println!("---\n# Serialized with curly_yaml\n{out}");
        dbg!(&data, &val, &de);
    }
    assert_eq!(val, de.ok().unwrap())
});
