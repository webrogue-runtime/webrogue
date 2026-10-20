set -ex
cd "$(dirname $0)"

sh ../../events-apigen/gen.sh

wit-bindgen c --out-dir ../../../webrogue-sdk/libraries/libwr4c webrogue-gfx.wit
