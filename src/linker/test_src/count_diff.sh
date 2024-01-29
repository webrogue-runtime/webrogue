cd $(dirname $0)

diff t1_my.dump t1_lld_strip.dump --suppress-common-lines --side-by-side | wc -l
