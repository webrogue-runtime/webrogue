const WEBROGUE_LICENSE: &str = include_str!("../../../LICENSE");

const APACHE_2_0: &str = include_str!("../licenses/apache-2.0.txt");
const APACHE_2_0_LLVM: &str = include_str!("../licenses/apache-2.0-llvm-extension.txt");
const LGPL_2_1: &str = include_str!("../licenses/lgpl-2.1.txt");
const GPL_3_0_GCC: &str = concat!(
    include_str!("../licenses/gpl-3.0.txt"),
    "\n",
    include_str!("../licenses/gcc-exception-for-gpl-3-0.txt")
);
const MIT: &str = include_str!("../licenses/mit.txt");
const BSD_2_CLAUSE: &str = include_str!("../licenses/bsd-2-clause.txt");
const MICROSOFT_CRT: &str = include_str!("../licenses/microsoft-clrt.txt");

const LICENSES: &[(&str, &str)] = &[
    ("webrogue", WEBROGUE_LICENSE),
    ("wasmtime", APACHE_2_0_LLVM),
    ("cranelift", APACHE_2_0_LLVM),
    ("llvm", APACHE_2_0_LLVM),
    ("gfxstream", APACHE_2_0),
    ("zstd", MIT),
    ("microsoft_crt", MICROSOFT_CRT),
    ("glibc", LGPL_2_1),
    ("musl", MIT),
    ("bionic", BSD_2_CLAUSE),
    ("libgcc", GPL_3_0_GCC),
];

pub fn get_licenses() -> impl Iterator<Item = (&'static str, &'static str)> {
    LICENSES.iter().copied()
}
