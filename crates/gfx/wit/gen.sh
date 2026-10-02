set -ex
cd "$(dirname $0)"

wit-bindgen c --out-dir ../../../webrogue-sdk/libraries/webroguegfx webrogue-gfx.wit