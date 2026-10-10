set -ex
cd "$(dirname $0)"

wit-bindgen c --out-dir ../../../webrogue-sdk/libraries/libwr4c webrogue-gfx.wit
