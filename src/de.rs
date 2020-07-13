use std::ops::{AddAssign, MulAssign};

use serde::{de, Deserialize};
use serde::de::IntoDeserializer;

pub struct Deserializer<R: std::io::Read> {
    reader: itertools::MultiPeek<xml::reader::Events<R>>,
    depth: i64,
    is_map_value: bool,
}

pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> crate::Result<T> {
    let conf = xml::ParserConfig::new()
        .trim_whitespace(true)
        .whitespace_to_characters(true)
        .replace_unknown_entity_references(true);
    let mut event_reader = xml::reader::EventReader::new_with_config(s.as_bytes(), conf);
    match event_reader.next()? {
        xml::reader::XmlEvent::StartDocument {
            version,
            encoding,
            standalone
        } => {
            trace!("start_document({:?}, {:?}, {:?})", version, encoding, standalone);
        },
        _ => return Err(crate::Error::ExpectedElement)
    }
    let mut deserializer = Deserializer {
        reader: itertools::multipeek(event_reader.into_iter()),
        depth: 0,
        is_map_value: false
    };
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<R: std::io::Read> Deserializer<R> {
    fn set_map_value(&mut self) {
        trace!("set_map_value()");
        self.is_map_value = true;
    }

    pub fn unset_map_value(&mut self) -> bool {
        std::mem::replace(&mut self.is_map_value, false)
    }

    fn peek(&mut self) -> crate::Result<&xml::reader::XmlEvent> {
        let next = match match self.reader.peek() {
            Some(n) => n,
            None => return Err(crate::Error::ExpectedElement)
        } {
            Ok(n) => n,
            Err(e) => return Err(e.into())
        };
        trace!("peek() -> {:?}", next);
        Ok(next)
    }

    fn reset_peek(&mut self) {
        self.reader.reset_peek();
    }

    fn next(&mut self) -> crate::Result<xml::reader::XmlEvent> {
        let next = match self.reader.next() {
            Some(n) => n,
            None => return Err(crate::Error::ExpectedElement)
        }?;
        match next {
            xml::reader::XmlEvent::StartElement { .. } => {
                self.depth += 1;
            }
            xml::reader::XmlEvent::EndElement { .. } => {
                self.depth -= 1;
            }
            _ => {}
        }
        trace!("next() -> {:?}; depth = {}", next, self.depth);
        Ok(next)
    }

    fn read_inner_value<T, F: FnOnce(&mut Self) -> crate::Result<T>>(&mut self, f: F) -> crate::Result<T> {
        trace!("read_inner_value()");
        if self.unset_map_value() {
            match self.next()? {
                xml::reader::XmlEvent::StartElement { name, .. } => {
                    let result = f(self)?;
                    self.expect_end_element(name)?;
                    Ok(result)
                },
                _ => Err(crate::Error::ExpectedElement)
            }
        } else {
            f(self)
        }
    }

    fn read_inner_value_attrs<T, F: FnOnce(&mut Self, Vec<xml::attribute::OwnedAttribute>) -> crate::Result<T>>(&mut self, f: F) -> crate::Result<T> {
        trace!("read_inner_value()");
        if self.unset_map_value() {
            match self.next()? {
                xml::reader::XmlEvent::StartElement { name, attributes, .. } => {
                    let result = f(self, attributes)?;
                    self.expect_end_element(name)?;
                    Ok(result)
                },
                _ => Err(crate::Error::ExpectedElement)
            }
        } else {
            f(self, vec![])
        }
    }

    fn expect_end_element(&mut self, old_name: xml::name::OwnedName) -> crate::Result<()> {
        trace!("expect_end_element({:?})", old_name);
        match self.next()? {
            xml::reader::XmlEvent::EndElement {name} => {
                if name == old_name {
                    Ok(())
                } else {
                    Err(crate::Error::ExpectedElement)
                }
            }
            _ => Err(crate::Error::ExpectedElement)
        }
    }

    fn parse_string(&mut self) -> crate::Result<String> {
        trace!("prase_string()");
        self.read_inner_value(|this| {
            match this.next()? {
                xml::reader::XmlEvent::CData(s) | xml::reader::XmlEvent::Characters(s) => {
                    Ok(s)
                }
                xml::reader::XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace
                } => {
                    let mut output: Vec<u8> = Vec::new();
                    let conf = xml::writer::EmitterConfig::new()
                        .perform_indent(false)
                        .write_document_declaration(false)
                        .normalize_empty_elements(true)
                        .cdata_to_characters(false)
                        .keep_element_names_stack(false)
                        .pad_self_closing(false);
                    let mut writer = conf.create_writer(&mut output);
                    writer.write(xml::writer::XmlEvent::StartElement {
                        name: name.borrow(),
                        attributes: attributes.iter().map(|a| a.borrow()).collect(),
                        namespace: std::borrow::Cow::Borrowed(&namespace)
                    }).unwrap();
                    let depth = this.depth - 1;
                    loop {
                        let event = this.next()?;
                        trace!("{:?}; {}; {}", event, this.depth, depth);
                        if this.depth == depth {
                            break;
                        }
                        if let Some(e) = event.as_writer_event() {
                            trace!("{:?}; {}; {}", event, this.depth, depth);
                            writer.write(e).unwrap();
                        }
                    }
                    writer.write(xml::writer::XmlEvent::EndElement {
                        name: Some(name.borrow())
                    }).unwrap();
                    Ok(String::from_utf8(output).unwrap())
                },
                _ => Err(crate::Error::ExpectedString)
            }
        })
    }

    fn parse_bool(&mut self) -> crate::Result<bool> {
        let s = self.parse_string()?;
        match s.to_lowercase().as_str() {
            "true" | "1" | "y" => Ok(true),
            "false" | "0" | "n" => Ok(false),
            _ => Err(crate::Error::ExpectedBool)
        }
    }

    fn parse_int<T: AddAssign<T> + MulAssign<T> + std::str::FromStr>(&mut self) -> crate::Result<T> {
        let s = self.parse_string()?;
        match s.parse::<T>() {
            Ok(i) => Ok(i),
            Err(_) => Err(crate::Error::ExpectedInt)
        }
    }
}

impl<'de, 'a, R: std::io::Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = crate::Error;

    fn deserialize_any<V: serde::de::Visitor<'de>>(self, _visitor: V) -> crate::Result<V::Value> {
        Err(crate::Error::Unsupported)
    }

    fn deserialize_bool<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f64<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_char<V: serde::de::Visitor<'de>>(self, _visitor: V) -> crate::Result<V::Value> {
        use std::str::FromStr;

        let s = self.parse_string()?;
        if s.len() == 1 {
            _visitor.visit_char(match char::from_str(&s) {
                Ok(c) => c,
                Err(_) => return Err(crate::Error::ExpectedChar)
            })
        } else {
            Err(crate::Error::ExpectedChar)
        }
    }

    fn deserialize_str<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_string<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: serde::de::Visitor<'de>>(self, _visitor: V) -> crate::Result<V::Value> {
        Err(crate::Error::Unsupported)
    }

    fn deserialize_byte_buf<V: serde::de::Visitor<'de>>(self, _visitor: V) -> crate::Result<V::Value> {
        Err(crate::Error::Unsupported)
    }

    fn deserialize_option<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_option()");
        if self.is_map_value {
            if let xml::reader::XmlEvent::StartElement { attributes, .. } = self.peek()? {
                if !attributes.is_empty() {
                    self.reset_peek();
                    return visitor.visit_some(self);
                }
            }
        }
        if let xml::reader::XmlEvent::EndElement { .. } = self.peek()? {
            if self.unset_map_value() {
                self.next()?;
            }
            self.next()?;
            visitor.visit_none()
        } else {
            self.reset_peek();
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: serde::de::Visitor<'de>>(self, name: &'static str, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_unit_struct({:?})", name);
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: serde::de::Visitor<'de>>(self, name: &'static str, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_newtype_struct({:?})", name);
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: serde::de::Visitor<'de>>(mut self, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_seq()");
        visitor.visit_seq(Seq::new(&mut self)?)
    }

    fn deserialize_tuple<V: serde::de::Visitor<'de>>(self, len: usize, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_tuple({:?})", len);
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: serde::de::Visitor<'de>>(self, name: &'static str, len: usize, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_tuple_struct({:?}, {:?})", name, len);
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_map()");
        self.read_inner_value_attrs(|this, attrs| {
            visitor.visit_map(Map::new(this, attrs, &[]))
        })
    }

    fn deserialize_struct<V: serde::de::Visitor<'de>>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_struct({:?}, {:?})", name, fields);
        self.read_inner_value_attrs(|this, attrs | {
            visitor.visit_map(Map::new(this, attrs, fields))
        })
    }

    fn deserialize_enum<V: serde::de::Visitor<'de>>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_enum({:?}, {:?})", name, variants);
        self.read_inner_value(|this| {
            visitor.visit_enum(Enum::new(this, variants))
        })
    }

    fn deserialize_identifier<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_identifier()");
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_ignored_any()");
        let depth = self.depth;
        loop {
            self.next()?;
            if self.depth == depth {
                break;
            }
        }
        visitor.visit_unit()
    }
}

struct Seq<'a, R: std::io::Read> {
    de: &'a mut Deserializer<R>,
    expected_name: Option<xml::name::OwnedName>,
}

impl<'a, R: std::io::Read> Seq<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> crate::Result<Self> {
        let name = if de.unset_map_value() {
            let val = match de.peek()? {
                xml::reader::XmlEvent::StartElement { name, .. } => {
                    Some(name.clone())
                },
                _ => return Err(crate::Error::ExpectedElement)
            };
            de.reset_peek();
            val
        } else {
            None
        };
        Ok(Self {
            de,
            expected_name: name
        })
    }
}

impl<'de, 'a, R: std::io::Read> de::SeqAccess<'de> for Seq<'a, R> {
    type Error = crate::Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> crate::Result<Option<T::Value>> {
        trace!("next_element_seed()");
        let more = match (self.de.peek()?, self.expected_name.as_ref()) {
            (xml::reader::XmlEvent::StartElement { ref name, .. }, Some(expected_name)) => {
                name == expected_name
            },
            (xml::reader::XmlEvent::EndElement { .. }, None) | (_, Some(_)) | (xml::reader::XmlEvent::EndDocument { .. }, _) => false,
            (_, None) => true,
        };
        self.de.reset_peek();
        if more {
            if self.expected_name.is_some() {
                self.de.set_map_value();
            }
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct Fields {
    fields: Vec<Field>,
    inner_value: bool
}

struct Field {
    namespace: Option<String>,
    local_name: String,
    name: String,
    attr: bool
}

impl From<&&str> for Field {
    fn from(from: &&str) -> Self {
        let mut attr = false;
        let from = if from.starts_with("$attr:") {
            attr = true;
            &from[6..]
        } else {
            from
        };
        let caps = crate::NAME_RE.captures(from).unwrap();
        let base_name = caps.name("e").unwrap().as_str().to_string();
        let namespace = caps.name("n").map(|n| n.as_str().to_string());
        Field {
            namespace,
            local_name: base_name,
            name: from.to_string(),
            attr
        }
    }
}

impl From<&[&str]> for Fields {
    fn from(from: &[&str]) -> Self {
        Fields {
            fields: from.iter().map(|f| f.into()).collect(),
            inner_value: from.contains(&"$value")
        }
    }
}

impl Fields {
    fn match_field(&self, name: &xml::name::OwnedName) -> String {
        for field in &self.fields {
            if field.local_name == name.local_name && field.namespace == name.namespace && !field.attr {
                trace!("match_field({:?}) -> {:?}", name, field.name);
                return field.name.clone()
            }
        }
        let name_str = if self.inner_value {
            "$value".to_string()
        } else {
            match &name.namespace {
                Some(n) => format!("{{{}}}{}", n, name.local_name),
                None => name.local_name.clone()
            }
        };
        trace!("match_field({:?}) -> {:?}", name, name_str);
        name_str
    }

    fn match_attr(&self, name: &xml::name::OwnedName) -> String {
        for field in &self.fields {
            if field.local_name == name.local_name && field.namespace == name.namespace && field.attr {
                let name_str = format!("$attr:{}", field.name);
                trace!("match_attr({:?}) -> {:?}", name, name_str);
                return name_str;
            }
        }
        let name_str = match &name.namespace {
            Some(n) => format!("{{{}}}{}", n, name.local_name),
            None => name.local_name.clone()
        };
        let name_str = format!("$attr:{}", name_str);
        trace!("match_attr({:?}) -> {:?}", name, name_str);
        name_str
    }
}

struct Map<'a, R: std::io::Read> {
    de: &'a mut Deserializer<R>,
    attrs: Vec<xml::attribute::OwnedAttribute>,
    fields: Fields,
    next_value: Option<String>,
    inner_value: bool
}

impl<'a, R: std::io::Read> Map<'a, R> {
    fn new(de: &'a mut Deserializer<R>, attrs: Vec<xml::attribute::OwnedAttribute>, fields: &[&str]) -> Self {
        Self {
            de,
            attrs,
            fields: fields.into(),
            next_value: None,
            inner_value: true
        }
    }
}

impl<'de, 'a, R: std::io::Read> de::MapAccess<'de> for Map<'a, R> {
    type Error = crate::Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(&mut self, seed: K) -> crate::Result<Option<K::Value>> {
        trace!("next_key_seed(); attrs = {:?}", self.attrs);
        match self.attrs.pop() {
            Some(xml::attribute::OwnedAttribute { name, value }) => {
                let name = self.fields.match_attr(&name);
                self.next_value = Some(value);
                seed.deserialize(name.as_str().into_deserializer()).map(Some)
            },
            None => {
                let val = match *self.de.peek()? {
                    xml::reader::XmlEvent::StartElement {
                        ref name, ..
                    } => {
                        let name = self.fields.match_field(name);
                        self.inner_value = name == "$value";
                        seed.deserialize(name.as_str().into_deserializer()).map(Some)
                    }
                    xml::reader::XmlEvent::Characters(_) | xml::reader::XmlEvent::CData(_) => {
                        seed.deserialize("$value".into_deserializer()).map(Some)
                    }
                    _ => Ok(None)
                };
                self.de.reset_peek();
                val
            }
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> crate::Result<V::Value> {
        trace!("next_value_seed(); next_value = {:?}", self.next_value);
        match self.next_value.take() {
            Some(val) => seed.deserialize(AttrValueDeserializer(val)),
            None => {
                if !std::mem::replace(&mut self.inner_value, false) {
                    self.de.set_map_value();
                }
                seed.deserialize(&mut *self.de)
            }
        }
    }
}

pub struct Enum<'a, R: std::io::Read> {
    de: &'a mut Deserializer<R>,
    fields: Fields,
}

impl<'a, R: std::io::Read> Enum<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>, fields: &[&str]) -> Self {
        Self {
            de,
            fields: fields.into()
        }
    }
}

impl<'de, 'a, R: std::io::Read> de::EnumAccess<'de> for Enum<'a, R> {
    type Error = crate::Error;
    type Variant = Self;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> crate::Result<(V::Value, Self::Variant)> {
        trace!("variant_seed()");
        let val = match self.de.peek()? {
            xml::reader::XmlEvent::StartElement {
                name, ..
            } => {
                let name_str = self.fields.match_field(name);
                let name_str: serde::de::value::StrDeserializer<crate::Error> = name_str.as_str().into_deserializer();
                self.de.set_map_value();
                Ok(seed.deserialize(name_str)?)
            },
            xml::reader::XmlEvent::Characters(s) | xml::reader::XmlEvent::CData(s) => {
                let name: serde::de::value::StrDeserializer<crate::Error> = s.as_str().into_deserializer();
                Ok(seed.deserialize(name)?)
            },
            _ => Err(crate::Error::ExpectedString)
        }?;
        self.de.reset_peek();
        Ok((val, self))
    }
}

impl<'de, 'a, R: std::io::Read> de::VariantAccess<'de> for Enum<'a, R> {
    type Error = crate::Error;

    fn unit_variant(self) -> crate::Result<()> {
        trace!("unit_variant()");
        self.de.unset_map_value();
        match self.de.next()? {
            xml::reader::XmlEvent::StartElement {
                name, attributes, ..
            } => if attributes.is_empty() {
                self.de.expect_end_element(name)
            } else {
                Err(crate::Error::ExpectedElement)
            },
            xml::reader::XmlEvent::Characters(_) | xml::reader::XmlEvent::CData(_) => Ok(()),
            _ => unreachable!()
        }
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> crate::Result<T::Value> {
        trace!("newtype_variant_seed()");
        seed.deserialize(self.de)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> crate::Result<V::Value> {
        trace!("tuple_variant({:?})", len);
        use serde::de::Deserializer;
        self.de.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> crate::Result<V::Value> {
        trace!("struct_variant({:?})", fields);
        use serde::de::Deserializer;
       self.de.deserialize_struct("", fields, visitor)
    }
}

struct AttrValueDeserializer(String);

macro_rules! deserialize_type_attr {
    ($deserialize:ident => $visit:ident) => {
        fn $deserialize<V: de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
            visitor.$visit(match self.0.parse() {
                Ok(v) => v,
                Err(_) => return Err(crate::Error::ExpectedInt)
            })
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for AttrValueDeserializer {
    type Error = crate::Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_string(self.0)
    }

    deserialize_type_attr!(deserialize_i8 => visit_i8);
    deserialize_type_attr!(deserialize_i16 => visit_i16);
    deserialize_type_attr!(deserialize_i32 => visit_i32);
    deserialize_type_attr!(deserialize_i64 => visit_i64);
    deserialize_type_attr!(deserialize_u8 => visit_u8);
    deserialize_type_attr!(deserialize_u16 => visit_u16);
    deserialize_type_attr!(deserialize_u32 => visit_u32);
    deserialize_type_attr!(deserialize_u64 => visit_u64);
    deserialize_type_attr!(deserialize_f32 => visit_f32);
    deserialize_type_attr!(deserialize_f64 => visit_f64);

    fn deserialize_enum<V: de::Visitor<'de>>(self, name: &str, variants: &'static [&'static str], visitor: V) -> crate::Result<V::Value> {
        trace!("deserialize_enum({:?}, {:?})", name, variants);
        visitor.visit_enum(self.0.into_deserializer())
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        visitor.visit_some(self)
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> crate::Result<V::Value> {
        match self.0.to_lowercase().as_str() {
            "true" | "1" | "y" => visitor.visit_bool(true),
            "false" | "0" | "n" => visitor.visit_bool(false),
            _ => Err(crate::Error::ExpectedBool),
        }
    }

    serde::forward_to_deserialize_any! {
        char str string unit seq bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple ignored_any byte_buf
    }
}