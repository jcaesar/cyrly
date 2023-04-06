use cyrly::CurlySerializer;
use serde::Serialize;
use serde_yaml::Value;
use std::{
    error::Error,
    io::{stdin, stdout, BufWriter},
};

fn main() -> Result<(), Box<dyn Error>> {
    serde_yaml::from_reader::<_, Value>(stdin())?.serialize(CurlySerializer::new(
        &mut cyrly::write::WriteEat(BufWriter::new(stdout())),
    ))?;
    Ok(())
}
