fn main() {
    use build_details::{BuildDetail, BuildDetails};
    use build_script_file_gen::gen_file_str;
    use rustc_version::*;
    use std::fmt::*;
    let v = version_meta().unwrap();
    let mut o = String::new();
    write!(o, "const RUSTC_VER: &str = \"{}\";", v.short_version_string).unwrap();
    gen_file_str("verinfo.rs", o.as_str());
    o.clear();
    BuildDetails::none()
        .include(BuildDetail::Timestamp)
        .include(BuildDetail::Version)
        .include(BuildDetail::Profile)
        .generate("build_details.rs")
        .unwrap();
}
