#![allow(unused_variables)]

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

struct ShortSinglelineEater(String, usize);
#[derive(Debug)]
struct Fallible;
impl std::fmt::Display for Fallible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "[error not stored]".fmt(f)
    }
}
impl std::error::Error for Fallible {}
impl ser::Error for Fallible {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self
    }
}
impl Eat for ShortSinglelineEater {
    type Error = Fallible;

    fn eat(&mut self, data: &str) -> Result<(), Self::Error> {
        let mut data = data.chars().peekable();
        while let Some(char) = data.next() {
            if self.1 == 0 {
                return Err(Fallible);
            }
            self.1 -= 1;
            if char == '\n' {
                while let Some(' ') = data.peek() {
                    data.next();
                }
                self.0.push(' ');
            } else {
                self.0.push(char);
            }
        }
        Ok(())
    }
}

impl<E: Eat> Serializer for CurlySerializer<'_, E> {
    type Ok = ();
    type Error = <E as Eat>::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

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
        self.glut.eat(&format!("{v:.1}"))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.glut.eat(&format!("{v:.1}"))
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

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
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
        name: &'static str,
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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.glut.eat("[\n")?;

        impl<E: Eat> SerializeSeq for CurlySerializer<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                self.indent(true)?;
                value.serialize(self.next_level())?;
                self.glut.eat(",\n")
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Self::end(self, "]")
            }
        }

        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.glut.eat("{\n")?;
        impl<E: Eat> SerializeMap for CurlySerializer<'_, E> {
            type Ok = ();

            type Error = <E as Eat>::Error;

            fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                self.indent(true)?;

                let mut singleline = ShortSinglelineEater(String::new(), 80);
                let res = key.serialize(CurlySerializer {
                    level: 0,
                    glut: &mut singleline,
                });
                let res = res.is_ok().then_some(singleline.0);
                if let Some(singleline) = res {
                    self.glut.eat(&singleline)?;
                } else {
                    self.glut.eat("? ")?;
                    key.serialize(self.next_level())?;
                    self.glut.eat("\n")?;
                    self.indent(true)?;
                }
                Ok(())
            }

            fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                self.glut.eat(": ")?;
                value.serialize(self.next_level())?;
                self.glut.eat(",\n")
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Self::end(self, "}")
            }
        }

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        impl<E: Eat> SerializeTuple for CurlySerializer<'_, E> {
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
        impl<E: Eat> SerializeTupleStruct for CurlySerializer<'_, E> {
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
        impl<E: Eat> SerializeTupleVariant for CurlySerializer<'_, E> {
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
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        impl<E: Eat> SerializeStruct for CurlySerializer<'_, E> {
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
        impl<E: Eat> SerializeStructVariant for CurlySerializer<'_, E> {
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
}

fn is_yaml_benign_str(v: &str) -> bool {
    v.chars().next().map_or(false, char::is_alphabetic)
        && v.chars().all(|v| v.is_ascii_alphanumeric() || v == '_')
}

fn is_yaml_special_str(v: &str) -> bool {
    matches!(
        v.to_ascii_lowercase().as_str(),
        "y" | "yes" | "n" | "no" | "true" | "false" | "on" | "off"
    )
}

impl<'e, E: Eat> CurlySerializer<'e, E> {
    pub fn new(glut: &'e mut E) -> Self {
        Self { level: 0, glut }
    }

    fn serialize_variant_name(
        &mut self,
        variant: &str,
    ) -> Result<(), <CurlySerializer<E> as Serializer>::Error> {
        self.glut.eat("!")?;
        self.glut.eat(&urlencoding::encode(variant))?;
        self.glut.eat(" ")?;
        Ok(())
    }

    fn indent(&mut self, extra: bool) -> Result<(), <CurlySerializer<E> as Serializer>::Error> {
        for _ in 0..self.level {
            self.glut.eat("  ")?;
        }
        Ok(if extra {
            self.glut.eat("  ")?;
        })
    }

    fn next_level(&mut self) -> CurlySerializer<E> {
        CurlySerializer {
            level: self.level + 1,
            glut: self.glut,
        }
    }

    fn end(mut self, arg: &str) -> Result<(), <CurlySerializer<E> as Serializer>::Error> {
        self.indent(false)?;
        self.glut.eat(arg)?;
        Ok(())
    }
}
