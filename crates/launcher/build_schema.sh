set -ex
cd $(dirname $0)

# npm install
rm -rf launcher-client
npx openapi-generator-cli generate -i launcher.yaml -g typescript-fetch -o launcher-client
find launcher-client -name "*.ts" -exec sed -i '1i// @ts-nocheck' {} \;
# sed -i "/import { mapValues } from '\.\.\/runtime';/d" launcher_proto_ts/*.ts
rm -rf out
node convert_to_json_schema.js

rm -rf src/openapi
npx openapi-generator-cli generate -i launcher.yaml -g rust-axum -o src/openapi --additional-properties=packageName=webrogue-launcher-server-openapi
