use clap::Args;
use schemars::{
    generate::SchemaSettings,
    transform::{transform_subschemas, Transform},
    Schema, SchemaGenerator,
};

#[derive(Clone)]
pub struct MyTransform;

impl Transform for MyTransform {
    fn transform(&mut self, schema: &mut Schema) {
        transform_subschemas(self, schema);
        let schema_object = schema.as_object_mut().unwrap();
        let Some(ty) = schema_object.get("type") else {
            return;
        };
        if !(ty
            .as_array()
            .cloned()
            .unwrap_or_default()
            .contains(&serde_json::Value::String("object".to_string()))
            || ty.as_str() == Some("object"))
        {
            return;
        }
        if !schema_object.contains_key("additionalProperties") {
            schema_object.insert(
                "additionalProperties".to_string(),
                serde_json::Value::Bool(false),
            );
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct SchemaDumpCommand {}
impl SchemaDumpCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let schema = SchemaGenerator::new(SchemaSettings::default().with_transform(MyTransform))
            .root_schema_for::<webrogue_wrapp::config::Config>();

        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }
}
