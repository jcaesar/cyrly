use std::println;

use alloc::{
    string::{String, ToString},
    vec,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

fn trips<S: Serialize + DeserializeOwned + std::fmt::Debug + PartialEq>(s: S) {
    println!(
        "---\n# Serialized with serde_yaml\n{}\n\n",
        serde_yaml::to_string(&s).unwrap()
    );

    let mut out = String::new();

    let ser = s.serialize(super::CurlySerializer::new(&mut out));

    println!("---\n# Serialized with curly_yaml\n{out}");

    ser.unwrap();
    let de = serde_yaml::from_str::<S>(&out).unwrap();
    assert_eq!(de, s);
}

#[test]
fn first() {
    trips(vec![1, 2, 3])
}

#[test]
fn second() {
    #[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
    struct Newtype(i64);
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Asdf {
        Bsdf,
        #[serde(rename = "this can't be an ident?")]
        Csdf {
            a: i32,
            b: i32,
        },
    }
    impl Default for Asdf {
        fn default() -> Self {
            Self::Csdf { a: 42, b: 13 }
        }
    }
    #[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
    struct Bar;
    #[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
    struct Foo {
        foo: (),
        bar: Bar,
        asdf: Asdf,
        newtype: Newtype,
        c: char,
    }

    trips(Foo::default())
}

#[test]
fn and_i_quote_null() {
    trips(serde_yaml::Value::String("null".to_string()))
}
