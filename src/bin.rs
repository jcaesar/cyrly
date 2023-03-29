use curly_yaml::{CurlySerializer, Eat};
use serde::Serialize;
use serde_yaml::Value;
use std::{
    error::Error,
    fmt::Display,
    io::{stdin, stdout, BufWriter, Write},
};

#[derive(Debug)]
struct DynError(Box<dyn Error>);
impl Display for DynError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl Error for DynError {}
impl serde::ser::Error for DynError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        DynError(Box::<dyn Error>::from(format!("{msg}")))
    }
}

struct Dia<W>(W);
impl<W: Write> Eat for Dia<W> {
    type Error = DynError;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        self.0
            .write_all(data.as_bytes())
            .map_err(|e| DynError(Box::new(e)))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    serde_yaml::from_reader::<_, Value>(stdin())?
        .serialize(CurlySerializer::new(&mut Dia(BufWriter::new(stdout()))))?;
    Ok(())
}
