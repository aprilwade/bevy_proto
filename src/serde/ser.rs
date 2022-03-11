use crate::components::ComponentList;
use crate::prelude::{Prototype, TemplateList};
use bevy::reflect::serde::ReflectSerializer;
use bevy::reflect::TypeRegistry;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

/// A serializer for [`Prototype`] data
pub struct PrototypeSerializer<'a> {
    prototype: &'a Prototype,
    registry: &'a TypeRegistry,
}

impl<'a> PrototypeSerializer<'a> {
    pub fn new(prototype: &'a Prototype, registry: &'a TypeRegistry) -> Self {
        Self {
            prototype,
            registry,
        }
    }
}

impl<'a> Serialize for PrototypeSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let total_entries = if self.prototype.templates.is_empty() {
            2
        } else {
            3
        };
        let mut state = serializer.serialize_map(Some(total_entries))?;
        state.serialize_entry("name", &self.prototype.name)?;
        if !self.prototype.templates.is_empty() {
            let template_serializer = TemplateListSerializer::new(&self.prototype.templates);
            state.serialize_entry("templates", &template_serializer)?;
        }

        let comp_list = &self.prototype.components;
        let comp_serializer = ComponentListSerializer::new(&comp_list, self.registry);

        state.serialize_entry("components", &comp_serializer)?;
        state.end()
    }
}

/// A custom serializer for [`ComponentList`] data
///
/// This can be used in your own custom [`Prototypical`](crate::Prototypical) struct to
/// easily serialize a list of components.
pub struct ComponentListSerializer<'a> {
    list: &'a ComponentList,
    registry: &'a TypeRegistry,
}

impl<'a> ComponentListSerializer<'a> {
    pub fn new(list: &'a ComponentList, registry: &'a TypeRegistry) -> Self {
        Self { list, registry }
    }
}

impl<'a> Serialize for ComponentListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.list.len()))?;
        let registry = self.registry.read();
        for value in self.list.iter() {
            let reflected_value = value.as_reflect();
            state.serialize_element(&ReflectSerializer::new(reflected_value, &registry))?;
        }
        state.end()
    }
}

pub struct TemplateListSerializer<'a> {
    list: &'a TemplateList,
}

impl<'a> TemplateListSerializer<'a> {
    pub fn new(list: &'a TemplateList) -> Self {
        Self { list }
    }
}

impl<'a> Serialize for TemplateListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for template in self.list.iter_defined_order() {
            seq.serialize_element(template)?;
        }
        seq.end()
    }
}