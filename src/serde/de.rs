use crate::components::ComponentList;
use crate::prelude::{Prototype, TemplateList};
use bevy::reflect::{serde::ReflectDeserializer, TypeRegistry};
use serde::de::value::SeqAccessDeserializer;
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

/// A deserializer for [`Prototype`] data
pub struct PrototypeDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> PrototypeDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for PrototypeDeserializer<'a> {
    type Value = Prototype;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProtoVisitor {
            registry: self.registry,
        })
    }
}

struct ProtoVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ProtoVisitor<'a> {
    type Value = Prototype;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a Prototype definition")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name = None;
        let mut templates = None;
        let mut components = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "name" => name = Some(map.next_value()?),
                "template" | "templates" => {
                    templates = Some(map.next_value_seed(TemplateListDeserializer)?)
                }
                "components" => {
                    components =
                        Some(map.next_value_seed(ComponentListDeserializer::new(self.registry))?)
                }
                invalid => return Err(A::Error::custom(format!("Invalid entry: {}", invalid))),
            }
        }

        Ok(Prototype {
            name: name.ok_or_else(|| A::Error::custom("Missing `name` property"))?,
            templates: templates.unwrap_or_default(),
            components: components.unwrap_or_default(),
        })
    }
}

/// A custom deserializer for [`ComponentList`] data
///
/// This can be used in your own custom [`Prototypical`](crate::Prototypical) struct to
/// easily deserialize a list of components.
pub struct ComponentListDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> ComponentListDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentListDeserializer<'a> {
    type Value = ComponentList;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ComponentListVisitor {
            registry: self.registry,
        })
    }
}

struct ComponentListVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ComponentListVisitor<'a> {
    type Value = ComponentList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("component list")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut list = Vec::default();
        let registry = self.registry.read();
        while let Some(value) = seq.next_element_seed(ReflectDeserializer::new(&registry))? {
            list.push(value);
        }
        ComponentList::from_reflected(&list, &self.registry).map_err(V::Error::custom)
    }
}

/// A custom deserializer for [`TemplateList`] data
///
/// This can be used in your own custom [`Prototypical`](crate::Prototypical) struct to
/// easily deserialize a list of templates.
///
/// For prototypes defined in YAML, a template list can take on the following forms:
///
/// * Inline List:
///   > ```yaml
///   > templates: [ A, B, C ]
///   > ```
/// * Multi-Line List:
///   > ```yaml
///   > templates:
///   >   - A
///   >   - B
///   >   - C
///   > ```
/// * Comma-Separated String:
///   > ```yaml
///   > templates: A, B, C # OR: "A, B, C"
///   > ```
///
/// > This also applies to other serialization formats: templates can be defined as either
/// > lists or comma-separated strings
pub struct TemplateListDeserializer;

impl<'de> DeserializeSeed<'de> for TemplateListDeserializer {
    type Value = TemplateList;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TemplateListVisitor)
    }
}

struct TemplateListVisitor;

impl<'de> Visitor<'de> for TemplateListVisitor {
    type Value = TemplateList;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("string or vec")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // Split string by commas
        // Allowing for: "A, B, C" to become [A, B, C]
        let list = v.split(',').map(|s| s.trim().to_string()).collect();
        Ok(TemplateList::new(list))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let list = Deserialize::deserialize(SeqAccessDeserializer::new(seq))?;
        Ok(TemplateList::new(list))
    }
}
