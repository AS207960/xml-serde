//! Custom serde XML serializer
//!
//! The special serde tag name `$value` equates to the inner value of an XML element.
//! Tags starting with `$attr:` will be encoded as attributes rather than new elements.
//! Namespaces and prefixes can be set using the tag name format `{namespace}prefix:tag-name`.

use serde::{ser, Serialize};

pub struct Serializer;

/// Serialise serde item to XML
///
/// # Arguments
/// * `value` - The value to be serialised
/// * `root` - The root XML element name
/// * `ns` - The default XML namespace
pub fn to_string<T>(value: &T) -> Result<String, crate::Error>
    where
        T: Serialize,
{
    let conf = xml::writer::EmitterConfig::new()
        .perform_indent(true)
        .write_document_declaration(true)
        .normalize_empty_elements(true)
        .cdata_to_characters(true)
        .keep_element_names_stack(true)
        .pad_self_closing(false);

    let c = std::io::Cursor::new(Vec::new());
    let mut writer = conf.create_writer(c);
    let mut serializer = Serializer;
    let val = value.serialize(&mut serializer)?;
    format_data(&mut writer, &val)?;
    Ok(String::from_utf8(writer.into_inner().into_inner()).unwrap())
}

#[derive(Debug)]
pub enum _SerializerData {
    CData(String),
    String(String),
    Seq(Vec<_SerializerData>),
    Struct { attrs: Vec<(String, String)>, contents: Vec<(String, _SerializerData)> },
}

impl _SerializerData {
    fn as_str(&self) -> String {
        match self {
            _SerializerData::CData(s) => s.clone(),
            _SerializerData::String(s) => s.clone(),
            _SerializerData::Seq(s) => s.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(","),
            _SerializerData::Struct { contents, .. } => contents.iter().map(|(_, d)| d.as_str()).collect::<Vec<_>>().join(","),
        }
    }
}

fn format_data<W: std::io::Write>(writer: &mut xml::EventWriter<W>, val: &_SerializerData) -> Result<(), crate::Error> {
    match val {
        _SerializerData::CData(s) => {
            writer.write(xml::writer::XmlEvent::cdata(s))?
        },
        _SerializerData::String(s) => {
            writer.write(xml::writer::XmlEvent::characters(s))?
        },
        _SerializerData::Seq(s) => {
            for d in s {
                format_data(writer, &d)?;
            }
        },
        _SerializerData::Struct {
            contents,
            ..
        } => {
            for (tag, d) in contents {
                if tag == "$value" {
                    format_data(writer, &d)?;
                } else {
                    let caps = crate::NAME_RE.captures(tag).unwrap();
                    let base_name = caps.name("e").unwrap().as_str();
                    let name = match caps.name("p") {
                        Some(p) => format!("{}:{}", p.as_str(), base_name),
                        None => base_name.to_string()
                    };

                    let attrs = match d {
                        _SerializerData::Struct {
                            attrs,
                            ..
                        } => attrs.to_owned(),
                        _ => vec![]
                    };
                    let attrs = attrs.iter().map(|(attr_k, attr_v)| {
                        let caps = crate::NAME_RE.captures(attr_k).unwrap();
                        let base_name = caps.name("e").unwrap().as_str();
                        let ns = caps.name("n").map(|n| n.as_str());
                        let prefix = caps.name("p").map(|n| n.as_str());
                        let name = xml::name::Name {
                            local_name: base_name,
                            namespace: ns,
                            prefix,
                        };
                        (name, attr_v)
                    }).collect::<Vec<_>>();
                    match d {
                        _SerializerData::Seq(s) => {
                            for d in s {
                                let mut elm = xml::writer::XmlEvent::start_element(name.as_str());
                                if let Some(n) = caps.name("n") {
                                    match caps.name("p") {
                                        Some(p) => elm = elm.ns(p.as_str(), n.as_str()),
                                        None => elm = elm.default_ns(n.as_str())
                                    };
                                }
                                for (name, attr_v) in attrs.clone() {
                                    elm = elm.attr(name, attr_v);
                                }

                                writer.write(elm)?;
                                format_data(writer, &d)?;
                                writer.write(xml::writer::XmlEvent::end_element())?;
                            }
                        },
                        d => {
                            let mut elm = xml::writer::XmlEvent::start_element(name.as_str());
                            if let Some(n) = caps.name("n") {
                                match caps.name("p") {
                                    Some(p) => elm = elm.ns(p.as_str(), n.as_str()),
                                    None => elm = elm.default_ns(n.as_str())
                                };
                            }
                            for (name, attr_v) in attrs {
                                elm = elm.attr(name, attr_v);
                            }

                            writer.write(elm)?;
                            format_data(writer, &d)?;
                            writer.write(xml::writer::XmlEvent::end_element())?;
                        }
                    };
                }
            }
        },
    }
    Ok(())
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = _SerializerData;
    type Error = crate::Error;
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a,>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = StructVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<_SerializerData, Self::Error> {
        let val = if v { "1" } else { "0" };
        Ok(_SerializerData::String(val.to_string()))
    }

    fn serialize_i8(self, v: i8) -> Result<_SerializerData, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<_SerializerData, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<_SerializerData, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::String(v.to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<_SerializerData, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<_SerializerData, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<_SerializerData, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::String(v.to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<_SerializerData, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::String(v.to_string()))
    }

    fn serialize_char(self, v: char) -> Result<_SerializerData, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::CData(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::String(hex::encode(v)))
    }

    fn serialize_none(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::String("".to_string()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<_SerializerData, Self::Error>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<_SerializerData, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<_SerializerData, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<_SerializerData, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<_SerializerData, Self::Error>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<_SerializerData, Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let value = value.serialize(&mut *self)?;
        Ok(_SerializerData::Struct {
            attrs: vec![],
            contents: vec![(variant.to_string(), value)]
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer {
            parent: self,
            output: vec![],
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            parent: self,
            keys: vec![],
            cur_key: String::new(),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer {
            parent: self,
            attrs: vec![],
            keys: vec![],
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructVariantSerializer {
            parent: self,
            attrs: vec![],
            keys: vec![],
            tag: variant.to_string(),
        })
    }
}

pub struct SeqSerializer<'a> {
    parent: &'a mut Serializer,
    output: Vec<_SerializerData>,
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        self.output.push(val);
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Seq(self.output))
    }
}

impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        self.output.push(val);
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Seq(self.output))
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        self.output.push(val);
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Seq(self.output))
    }
}

impl<'a> ser::SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        self.output.push(val);
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Seq(self.output))
    }
}

pub struct MapSerializer<'a> {
    parent: &'a mut Serializer,
    keys: Vec<(String, _SerializerData)>,
    cur_key: String,
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = key.serialize(&mut *self.parent)?;
        self.cur_key = val.as_str();
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        self.keys.push((self.cur_key.to_owned(), val));
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Struct {
            attrs: vec![],
            contents: self.keys,
        })
    }
}

pub struct StructSerializer<'a> {
    parent: &'a mut Serializer,
    attrs: Vec<(String, String)>,
    keys: Vec<(String, _SerializerData)>,
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        if key.starts_with("$attr:") {
            self.attrs.push((key[6..].to_string(), val.as_str()));
        } else {
            self.keys.push((key.to_string(),val));
        }
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Struct {
            attrs: self.attrs,
            contents: self.keys,
        })
    }
}

pub struct StructVariantSerializer<'a> {
    parent: &'a mut Serializer,
    attrs: Vec<(String, String)>,
    keys: Vec<(String, _SerializerData)>,
    tag: String,
}

impl<'a> ser::SerializeStructVariant for StructVariantSerializer<'a> {
    type Ok = _SerializerData;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
    {
        let val = value.serialize(&mut *self.parent)?;
        if key.starts_with("$attr:") {
            self.attrs.push((key[6..].to_string(), val.as_str()));
        } else {
            self.keys.push((key.to_string(),val));
        }
        Ok(())
    }

    fn end(self) -> Result<_SerializerData, Self::Error> {
        Ok(_SerializerData::Struct {
            attrs: vec![],
            contents: vec![(self.tag, _SerializerData::Struct {
                attrs: self.attrs,
                contents: self.keys,
            })]
        })
    }
}
