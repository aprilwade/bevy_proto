mod de;
mod ser;

pub use de::{ComponentListDeserializer, PrototypeDeserializer, TemplateListDeserializer};
pub use ser::{ComponentListSerializer, PrototypeSerializer};

#[cfg(test)]
pub(crate) mod tests {
    use super::{PrototypeDeserializer, PrototypeSerializer};
    use crate::prelude::{
        ComponentList, ProtoComponent, Prototype, ReflectProtoComponent, TemplateList,
    };
    use bevy::prelude::{Component, Reflect};
    use bevy::reflect::{FromReflect, TypeRegistry, TypeRegistryArc};
    use serde::de::DeserializeSeed;
    use serde::{Deserialize, Serialize};
    use serde_yaml::Error;

    #[derive(Reflect, FromReflect, Component, ProtoComponent, Clone)]
    #[reflect(ProtoComponent)]
    pub struct MyComponent {
        foo: usize,
        bar: Option<Name>,
    }

    // `Serialize + Deserialize` required since it's stored as an `Option<Name>` in `MyComponent`
    #[derive(Reflect, FromReflect, Clone, Serialize, Deserialize)]
    pub struct Name {
        x: String,
    }

    fn setup() -> (Prototype, TypeRegistryArc) {
        let prototype = Prototype {
            name: String::from("Foo"),
            templates: TemplateList::new(vec![String::from("IFoo"), String::from("IBar")]),
            components: ComponentList::new(vec![Box::new(MyComponent {
                foo: 123,
                bar: Some(Name {
                    x: String::from("hello"),
                }),
            })]),
        };

        let registry = TypeRegistry::default();
        registry.write().register::<usize>();
        registry.write().register::<i32>();
        registry.write().register::<String>();
        registry.write().register::<MyComponent>();
        registry.write().register::<Name>();
        registry.write().register::<Option<Name>>();

        (prototype, registry)
    }

    fn serialize(proto: &Prototype, registry: &TypeRegistry) -> serde_yaml::Result<String> {
        let serializer = PrototypeSerializer::new(&proto, &registry);
        serde_yaml::to_string(&serializer)
    }

    fn deserialize(data: &str, registry: &TypeRegistry) -> Result<Prototype, Error> {
        let deserializer = PrototypeDeserializer::new(&registry);
        let de = serde_yaml::Deserializer::from_str(&data);
        deserializer.deserialize(de)
    }

    const DATA: &str = r#"---
name: Foo
templates:
  - IFoo
  - IBar
components:
  - type: "bevy_proto::serde::tests::MyComponent"
    struct:
      foo:
        type: usize
        value: 123
      bar:
        type: "core::option::Option<bevy_proto::serde::tests::Name>"
        value:
          x: hello
"#;

    #[test]
    fn should_serialize() -> anyhow::Result<()> {
        let (proto, registry) = setup();
        let expected = DATA;
        let output = serialize(&proto, &registry)?;
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn should_deserialize() -> anyhow::Result<()> {
        let (expected, registry) = setup();

        let output = deserialize(DATA, &registry)?;
        assert_eq!(expected, output);

        Ok(())
    }
}
