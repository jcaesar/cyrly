#![forbid(unsafe_code)]

#[cfg(test)]
mod test;

use serde::{
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serializer,
};

pub struct CurlySerializer<'a, E> {
    level: usize,
    pub multiline: bool,
    glut: &'a mut E,
}

pub trait Eat {
    type Error: ser::Error;
    fn eat(&mut self, data: &str) -> Result<(), Self::Error>;
}

impl Eat for String {
    type Error = std::fmt::Error;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        self.push_str(data);
        Ok(())
    }
}

pub struct WriteEat<T>(T);
impl<T: std::io::Write> Eat for WriteEat<T> {
    type Error = WriteEatError;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        self.0.write_all(data.as_bytes()).map_err(WriteEatError)
    }
}
#[derive(Debug)]
pub struct WriteEatError(std::io::Error);
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

struct ShortEater(String, usize);
#[derive(Debug)]
struct Fallible;
impl std::fmt::Display for Fallible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "[error not stored]".fmt(f)
    }
}
impl std::error::Error for Fallible {}
impl ser::Error for Fallible {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self
    }
}
impl Eat for ShortEater {
    type Error = Fallible;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        for char in data.chars() {
            if self.1.checked_sub(1).map(|u| self.1 = u).is_none() {
                return Err(Fallible);
            }
            self.0.push(char);
        }
        Ok(())
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

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&v.to_string())
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

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if is_yaml_special_str(v) {
            self.glut.eat("\"")?;
            self.glut.eat(v)?;
            self.glut.eat("\"")?;
        } else if is_yaml_benign_str(v) {
            self.glut.eat(v)?;
        } else {
            self.glut.eat("\"")?;
            for c in v.chars() {
                match c {
                    '\\' => self.glut.eat("\\\\")?,
                    '"' => self.glut.eat("\\\"")?,
                    '\r' => self.glut.eat("\\r")?,
                    '\n' => self.glut.eat("\\n")?,
                    c if c.is_ascii_graphic() || c.is_alphanumeric() => {
                        self.glut.eat(c.encode_utf8(&mut [0u8; 4]))?;
                    }
                    c => match &*c.encode_utf16(&mut [0u16; 2]) {
                        &[tb] => {
                            self.glut.eat("\\u")?;
                            self.glut.eat(&format!("{:04x}", tb))?;
                        }
                        _ => {
                            self.glut.eat("\\U")?;
                            self.glut.eat(&format!("{:08x}", c as u32))?;
                        }
                    },
                }
            }
            self.glut.eat("\"")?;
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        v.iter()
            .try_for_each(|b| SerializeSeq::serialize_element(&mut seq, b))?;
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.glut.eat("null")
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
    pub fn new(glut: &'e mut E) -> Self {
        Self {
            level: 0,
            multiline: true,
            glut,
        }
    }

    fn serialize_variant_name(&mut self, variant: &str) -> Result<(), <E as Eat>::Error> {
        self.glut.eat("!")?;
        self.glut.eat(&urlencoding::encode(variant))?;
        self.glut.eat(" ")?;
        Ok(())
    }

    fn indent(&mut self, extra: bool) -> Result<(), <E as Eat>::Error> {
        if self.multiline {
            self.glut.eat("\n")?;
            for _ in 0..self.level {
                self.glut.eat("  ")?;
            }
            if extra {
                self.glut.eat("  ")?;
            }
        } else {
            self.glut.eat(" ")?;
        }
        Ok(())
    }

    fn next_level(&mut self) -> CurlySerializer<E> {
        CurlySerializer {
            level: self.level + 1,
            multiline: self.multiline,
            glut: self.glut,
        }
    }

    pub fn multiline(self) -> Self {
        CurlySerializer {
            level: self.level,
            multiline: true,
            glut: self.glut,
        }
    }

    pub fn oneline(self) -> Self {
        CurlySerializer {
            level: self.level,
            multiline: false,
            glut: self.glut,
        }
    }

    fn start(&mut self, start: &str) -> Result<(), <E as Eat>::Error> {
        self.glut.eat(start)?;
        Ok(())
    }

    fn end(mut self, arg: &str, empty: bool) -> Result<(), <E as Eat>::Error> {
        if !empty {
            self.indent(false)?;
        }
        self.glut.eat(arg)?;
        Ok(())
    }

    fn serialize_float(
        self,
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
        self.glut.eat(s)
    }
}

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
            false => self.ser.glut.eat(",")?,
        }
        self.ser.indent(true)?;
        value.serialize(self.ser.next_level())?;
        if self.ser.multiline {
            self.ser.glut.eat(",")?;
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
            false => self.ser.glut.eat(",")?,
        }
        self.ser.indent(true)?;

        if self.ser.multiline {
            let mut short = ShortEater(String::new(), 80);
            let res = key.serialize(CurlySerializer {
                glut: &mut short,
                multiline: false,
                level: self.ser.level,
            });
            let res = res.is_ok().then_some(short.0);
            if let Some(singleline) = res {
                self.ser.glut.eat(&singleline)?;
            } else {
                self.ser.glut.eat("? ")?;
                key.serialize(self.ser.next_level())?;
                self.ser.indent(true)?;
            }
        } else {
            key.serialize(self.ser.next_level())?;
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.next(MapNext::Value)?;
        self.ser.glut.eat(": ")?;
        value.serialize(self.ser.next_level())?;
        if self.ser.multiline {
            self.ser.glut.eat(",")?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        CurlySerializer::end(self.ser, "}", self.first)
    }
}
