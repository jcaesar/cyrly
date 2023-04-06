#![doc = include_str!("./README.md")]
#![forbid(unsafe_code)]
#![no_std]

#[cfg(test)]
mod test;

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::{
    format,
    string::{String, ToString},
};
use serde::{
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

/// Serialize the given data structure as a string
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, core::fmt::Error> {
    let mut out = String::new();
    value.serialize(CurlySerializer::new(&mut out))?;
    Ok(out)
}

#[cfg(feature = "std")]
/// Serialize the given data structure into the stream
///
/// Note: the output will be written in small chunks, often single bytes.
/// This can lead to terrible performance when the write is directly flushed to the operating system.
/// If in doubt, use a [BufWriter][std::io::BufWriter].
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<(), std::io::Error>
where
    W: std::io::Write,
    T: ?Sized + Serialize,
{
    value
        .serialize(CurlySerializer::new(&mut write::WriteEat(writer)))
        .map_err(|write::WriteEatError(e)| e)
}

/// Main serializer implementation
///
/// Note that this serializer produces YAML tags for enums, e.g. `enum Foo { Bar(i32) }` will result in `!Bar 42`.
/// See [serde_yaml::with](https://docs.rs/serde_yaml/latest/serde_yaml/with/index.html) for configuration options.
pub struct CurlySerializer<'a, E> {
    /// Use more than one line (defaults to true in `new`)
    pub multiline: bool,
    level: usize,
    glut: &'a mut E,
    max_output: Option<&'a mut usize>,
}

/// Helper trait for data output from serializer
///
/// Normally, you can just rely on [to_string] or [to_writer]
pub trait Eat {
    /// Error shared with [CurlySerializer]
    type Error: ser::Error;
    /// Output some tokens
    fn eat(&mut self, data: &str) -> Result<(), Self::Error>;
}

impl Eat for String {
    type Error = core::fmt::Error;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        self.push_str(data);
        Ok(())
    }
}

#[cfg(feature = "std")]
/// Adaptors for [std::io::Write]
pub mod write {
    extern crate std;
    use super::*;

    /// Write wrapper
    pub struct WriteEat<T>(pub T);
    impl<T: std::io::Write> Eat for WriteEat<T> {
        type Error = WriteEatError;

        fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
            self.0.write_all(data.as_bytes()).map_err(WriteEatError)
        }
    }

    /// Error wrapper
    #[derive(Debug)]
    pub struct WriteEatError(pub std::io::Error);
    impl std::fmt::Display for WriteEatError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    impl std::error::Error for WriteEatError {}
    impl serde::ser::Error for WriteEatError {
        fn custom<T>(msg: T) -> Self
        where
            T: std::fmt::Display,
        {
            WriteEatError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{msg}"),
            ))
        }
    }
}

impl<'e, E: Eat> Serializer for CurlySerializer<'e, E> {
    type Ok = ();
    type Error = <E as Eat>::Error;
    type SerializeSeq = CurlySeq<'e, E>;
    type SerializeTuple = CurlySeq<'e, E>;
    type SerializeTupleStruct = CurlySeq<'e, E>;
    type SerializeTupleVariant = CurlySeq<'e, E>;
    type SerializeMap = CurlyMap<'e, E>;
    type SerializeStruct = CurlyMap<'e, E>;
    type SerializeStructVariant = CurlyMap<'e, E>;

    fn collect_str<T: core::fmt::Display + ?Sized>(
        self,
        v: &T,
    ) -> Result<<Self as Serializer>::Ok, <Self as Serializer>::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.eat(&v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_float(
            v.is_nan(),
            v == f32::INFINITY,
            v == f32::NEG_INFINITY,
            |buf| buf.format_finite(v),
        )
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_float(
            v.is_nan(),
            v == f64::INFINITY,
            v == f64::NEG_INFINITY,
            |buf| buf.format_finite(v),
        )
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&[v].iter().collect::<String>())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        if is_yaml_special_str(v) {
            self.eat("\"")?;
            self.eat(v)?;
            self.eat("\"")?;
        } else if self.multiline {
            if let Some(shawt) = self.serialize_short(v, 80) {
                self.eat(&shawt)?;
            } else {
                self.eat("\"")?;
                let mut chars_on_line = usize::MAX;
                let mut toks = WordOrSpace(v).peekable();
                while let Some(tok) = toks.next() {
                    match tok {
                        " " => {
                            // Newline "early" if the next word wouldn't fit onto the current line - but only up to some sensible length
                            // Working with length here is a crude approximation
                            let next_len = toks
                                .peek()
                                .map(|s| s.len())
                                .filter(|&l| l < 60)
                                .unwrap_or(0);
                            if chars_on_line.saturating_add(next_len) >= 80 {
                                assert!(self.multiline);
                                chars_on_line = 0;
                                self.indent(false)?;
                                match toks.peek() {
                                    Some(&" ") => {
                                        self.eat(" \\ ")?;
                                        toks.next();
                                    }
                                    _ => self.eat("  ")?,
                                }
                            } else {
                                match toks.peek() {
                                    Some(&" ") => {
                                        self.eat(" ")?;
                                        while toks.peek() == Some(&" ") {
                                            self.eat(" ")?;
                                            if chars_on_line >= 80 {
                                                self.eat("\\")?;
                                                break;
                                            }
                                            chars_on_line += 1;
                                            toks.next();
                                        }
                                    }
                                    Some(_) => self.eat(tok)?,
                                    None => self.indent(false)?,
                                }
                            }
                        }
                        "\n" => {
                            self.eat("\\n")?;
                            chars_on_line = usize::MAX;
                            match toks.peek() {
                                Some(&"\n") => {
                                    self.eat("\\")?;
                                    self.indent(true)?
                                }
                                None => {
                                    self.eat("\\")?;
                                    self.indent(false)?
                                }
                                _ => (),
                            }
                        }
                        a => {
                            for c in a.chars() {
                                if chars_on_line >= 80 {
                                    self.eat("\\")?;
                                    self.indent(true)?;
                                    chars_on_line = 0;
                                }
                                assert!(c != ' ');
                                self.serialize_char_in_string(c)?;
                                chars_on_line += 1;
                            }
                            if toks.peek().is_none() {
                                self.eat("\\")?;
                                self.indent(false)?;
                            }
                        }
                    }
                }
                self.eat("\"")?;
            }
        } else {
            if is_yaml_benign_str(v) {
                self.eat(v)?;
            } else {
                self.eat("\"")?;
                for c in v.chars() {
                    self.serialize_char_in_string(c)?;
                }
                self.eat("\"")?;
            }
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        v.iter()
            .try_for_each(|b| SerializeSeq::serialize_element(&mut seq, b))?;
        SerializeSeq::end(seq)
    }

    fn serialize_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.eat("null")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.serialize_variant_name(variant)?;
        value.serialize(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        impl<E: Eat> SerializeTuple for CurlySeq<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                SerializeSeq::serialize_element(self, value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                SerializeSeq::end(self)
            }
        }

        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        impl<E: Eat> SerializeTupleStruct for CurlySeq<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                SerializeTuple::serialize_element(self, value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                SerializeTuple::end(self)
            }
        }

        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        impl<E: Eat> SerializeTupleVariant for CurlySeq<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                SerializeTuple::serialize_element(self, value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                SerializeTuple::end(self)
            }
        }

        self.serialize_variant_name(variant)?;
        self.serialize_tuple(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        impl<E: Eat> SerializeStruct for CurlyMap<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_field<T: ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                SerializeMap::serialize_key(self, key)?;
                SerializeMap::serialize_value(self, value)?;
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                SerializeMap::end(self)
            }
        }

        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        mut self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        impl<E: Eat> SerializeStructVariant for CurlyMap<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_field<T: ?Sized>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                SerializeStruct::serialize_field(self, key, value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                SerializeStruct::end(self)
            }
        }

        self.serialize_variant_name(variant)?;
        self.serialize_struct(name, len)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        CurlySeq::new(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        CurlyMap::new(self)
    }
}

fn is_yaml_benign_str(v: &str) -> bool {
    v.chars().next().map_or(false, char::is_alphabetic)
        && v.chars().all(|v| v.is_ascii_alphanumeric() || v == '_')
}

fn is_yaml_special_str(v: &str) -> bool {
    matches!(
        v.to_ascii_lowercase().as_str(),
        "y" | "yes" | "n" | "no" | "true" | "false" | "on" | "off" | "null" | "nan" | "inf"
    )
}

impl<'e, E: Eat> CurlySerializer<'e, E> {
    /// Create a new instance.
    pub fn new(glut: &'e mut E) -> Self {
        Self {
            level: 0,
            multiline: true,
            glut,
            max_output: None,
        }
    }

    fn serialize_variant_name(&mut self, variant: &str) -> Result<(), <E as Eat>::Error> {
        self.eat("!")?;
        self.eat(&urlencoding::encode(variant))?;
        self.eat(" ")?;
        Ok(())
    }

    fn indent(&mut self, extra: bool) -> Result<(), <E as Eat>::Error> {
        if self.multiline {
            self.eat("\n")?;
            for _ in 0..self.level {
                self.eat("  ")?;
            }
            if extra {
                self.eat("  ")?;
            }
        } else {
            self.eat(" ")?;
        }
        Ok(())
    }

    fn next_level(&mut self) -> CurlySerializer<E> {
        CurlySerializer {
            level: self.level + 1,
            multiline: self.multiline,
            glut: self.glut,
            max_output: self.max_output.as_deref_mut(),
        }
    }

    /// Convenience function. Builders are overrated.
    pub fn multiline(self) -> Self {
        CurlySerializer {
            multiline: true,
            ..self
        }
    }

    /// Convenience function. Builders are overrated.
    pub fn oneline(self) -> Self {
        CurlySerializer {
            multiline: false,
            ..self
        }
    }

    fn start(&mut self, start: &str) -> Result<(), <E as Eat>::Error> {
        self.eat(start)?;
        Ok(())
    }

    fn end(mut self, arg: &str, empty: bool) -> Result<(), <E as Eat>::Error> {
        if !empty {
            self.indent(false)?;
        }
        self.eat(arg)?;
        Ok(())
    }

    fn serialize_float(
        mut self,
        is_nan: bool,
        infinity: bool,
        neg_infinity: bool,
        v: impl Fn(&mut ryu::Buffer) -> &str,
    ) -> Result<(), <E as Eat>::Error> {
        let mut buf = ryu::Buffer::new();
        let s = match (is_nan, infinity, neg_infinity) {
            (true, false, false) => ".nan",
            (false, true, false) => ".inf",
            (false, false, true) => "-.inf",
            (false, false, false) => v(&mut buf),
            _ => unreachable!(),
        };
        self.eat(s)
    }

    fn serialize_short<T: Serialize + ?Sized>(
        &mut self,
        key: &T,
        max_len: usize,
    ) -> Option<String> {
        let max_len = match self.max_output {
            Some(&mut max_output) if max_output < max_len && !self.multiline => return None,
            Some(&mut max_output) if max_output < max_len => max_output,
            _ => max_len,
        };
        let mut max_short_output = max_len;
        let mut short = String::with_capacity(max_len);
        let res = key.serialize(CurlySerializer {
            glut: &mut short,
            multiline: false,
            level: self.level,
            max_output: Some(&mut max_short_output),
        });
        let res = res.is_ok().then_some(short)?;
        assert!(res.len() <= max_len);
        Some(res)
    }

    fn serialize_char_in_string(&mut self, c: char) -> Result<(), <E as Eat>::Error> {
        Ok(match c {
            '\0' => self.eat("\\0")?,
            '\\' => self.eat("\\\\")?,
            '"' => self.eat("\\\"")?,
            '\r' => self.eat("\\r")?,
            '\n' => self.eat("\\n")?,
            ' ' => self.eat(" ")?,
            c if c.is_ascii_graphic() || c.is_alphanumeric() => {
                self.eat(c.encode_utf8(&mut [0u8; 4]))?;
            }
            c => match &*c.encode_utf16(&mut [0u16; 2]) {
                &[tb] => {
                    self.eat("\\u")?;
                    self.eat(&format!("{:04x}", tb))?;
                }
                _ => {
                    self.eat("\\U")?;
                    self.eat(&format!("{:08x}", c as u32))?;
                }
            },
        })
    }

    fn eat(&mut self, v: &str) -> Result<(), <E as Eat>::Error> {
        if let Some(max_len) = self.max_output.as_mut() {
            if v.len() > **max_len {
                return Err(ser::Error::custom("internal: length exceeded"));
            }
            **max_len = max_len.saturating_sub(v.len());
        }
        self.glut.eat(v)?;
        Ok(())
    }
}

#[doc(hidden)]
pub struct CurlySeq<'a, E> {
    first: bool,
    ser: CurlySerializer<'a, E>,
}
impl<'e, E: Eat> CurlySeq<'e, E> {
    fn new(mut ser: CurlySerializer<'e, E>) -> Result<Self, <E as Eat>::Error> {
        CurlySerializer::start(&mut ser, "[")?;
        Ok(CurlySeq { first: true, ser })
    }
}
impl<E: Eat> SerializeSeq for CurlySeq<'_, E> {
    type Ok = ();

    type Error = <E as Eat>::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        match self.ser.multiline || self.first {
            true => self.first = false,
            false => self.ser.eat(",")?,
        }
        self.ser.indent(true)?;
        value.serialize(self.ser.next_level())?;
        if self.ser.multiline {
            self.ser.eat(",")?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        CurlySerializer::end(self.ser, "]", self.first)
    }
}

#[derive(PartialEq, Clone, Copy)]
enum MapNext {
    Key,
    Value,
}
#[doc(hidden)]
pub struct CurlyMap<'e, E> {
    next: MapNext,
    first: bool,
    ser: CurlySerializer<'e, E>,
}
impl<'e, E: Eat> CurlyMap<'e, E> {
    fn new(mut ser: CurlySerializer<'e, E>) -> Result<Self, <E as Eat>::Error> {
        CurlySerializer::start(&mut ser, "{")?;
        Ok(CurlyMap {
            first: true,
            next: MapNext::Key,
            ser,
        })
    }
    fn next(&mut self, next: MapNext) -> Result<(), <E as Eat>::Error> {
        use MapNext::*;
        match (self.next, next) {
            (Key, Value) => self.serialize_key(&())?,
            (Value, Key) => self.serialize_value(&())?,
            _ => (),
        }
        match next {
            Key => self.next = Value,
            Value => self.next = Key,
        }
        Ok(())
    }
}

impl<E: Eat> SerializeMap for CurlyMap<'_, E> {
    type Ok = ();

    type Error = <E as Eat>::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.next(MapNext::Key)?;
        match self.ser.multiline || self.first {
            true => self.first = false,
            false => self.ser.eat(",")?,
        }
        self.ser.indent(true)?;

        let shortlen = match self.ser.multiline {
            true => 80,   // On multiline,
            false => 512, // On singliline, YAML 1.1 forbids flow keys longer than 1024 without "?". Approximate.
        };
        if let Some(singleline) = self.ser.serialize_short(key, shortlen) {
            self.ser.eat(&singleline)?;
        } else {
            if self.ser.multiline
                || self
                    .ser
                    .max_output
                    .as_ref()
                    .map_or(true, |x| **x > shortlen)
            {
                self.ser.eat("? ")?;
            }
            key.serialize(self.ser.next_level())?;
            if self.ser.multiline {
                self.ser.indent(true)?;
            }
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.next(MapNext::Value)?;
        self.ser.eat(": ")?;
        value.serialize(self.ser.next_level())?;
        if self.ser.multiline {
            self.ser.eat(",")?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        CurlySerializer::end(self.ser, "}", self.first)
    }
}

struct WordOrSpace<'a>(&'a str);

impl<'a> Iterator for WordOrSpace<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.0.char_indices();
        match chars.next() {
            None => None,
            Some((_, c)) if c.is_whitespace() => match chars.next() {
                Some((i, _)) => {
                    let ret = &self.0[..i];
                    self.0 = &self.0[i..];
                    return Some(ret);
                }
                None => {
                    let ret = self.0;
                    self.0 = "";
                    return Some(ret);
                }
            },
            _ => {
                while let Some((i, c)) = chars.next() {
                    if c.is_whitespace() {
                        let ret = &self.0[..i];
                        self.0 = &self.0[i..];
                        return Some(ret);
                    }
                }
                let ret = self.0;
                self.0 = "";
                return Some(ret);
            }
        }
    }
}
