const fs = require('fs');
const yaml = require('js-yaml');

const openApiSpec = yaml.load(fs.readFileSync('launcher.yaml', 'utf8'));

const webrogueConfigSchema = openApiSpec.components.schemas.WebrogueConfig;

function resolveRef(ref, root = openApiSpec) {
    if (ref.startsWith('#/')) {
        const path = ref.slice(2).split('/');
        let current = root;
        for (const segment of path) {
            current = current[segment];
        }
        return current;
    }
    return null;
}

function convertToJsonSchema(openApiSchema, root = openApiSpec) {
    if (openApiSchema.$ref) {
        const resolved = resolveRef(openApiSchema.$ref, root);
        if (resolved) {
            return convertToJsonSchema(resolved, root);
        }
    }

    const jsonSchema = { ...openApiSchema };

    delete jsonSchema.$ref;

    if (jsonSchema.properties) {
        for (const [propName, propSchema] of Object.entries(jsonSchema.properties)) {
            jsonSchema.properties[propName] = convertToJsonSchema(propSchema, root);
        }
    }

    if (jsonSchema.items) {
        jsonSchema.items = convertToJsonSchema(jsonSchema.items, root);
    }

    if (jsonSchema.additionalProperties) {
        if (typeof jsonSchema.additionalProperties === 'object') {
            jsonSchema.additionalProperties = convertToJsonSchema(jsonSchema.additionalProperties, root);
        }
    }

    return jsonSchema;
}

const jsonSchema = convertToJsonSchema(webrogueConfigSchema);

jsonSchema.$schema = "https://json-schema.org/draft/2020-12/schema";
jsonSchema.title = "WebrogueConfig";
jsonSchema.description = "Webrogue application configuration schema";

fs.writeFileSync('wrapp-config.schema.json', JSON.stringify(jsonSchema, null, 4));
