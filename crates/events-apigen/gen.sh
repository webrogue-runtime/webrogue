set -ex
cd "$(dirname $0)"

cargo run -- ../../webrogue-sdk/libraries/webroguegfx/webroguegfx.h ../../webrogue-sdk/libraries/webroguegfx/webroguegfx_events.c
