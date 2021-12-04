fn main() {
    use build_script_file_gen::gen_file_str;
    use compression::prelude::*;
    use pciids::*;
    use std::fmt::Write;
    use std::fs::read;
    use std::io::Cursor;
    println!(
        "cargo:rerun-if-changed={}/pci.ids.gz",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut o = String::with_capacity(1 << 20);
    let bytes = read(format!("{}/pci.ids.gz", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
        .iter()
        .cloned()
        .decode(&mut GZipDecoder::new())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut r = Cursor::new(bytes);
    let mut idsdb = PciIdData::new();
    idsdb.add_pci_ids_data(&mut r).unwrap();
    writeln!(
        o,
        "#[inline]\nconst fn classify_class(class: u8) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(o, "match class {{").unwrap();
    (0..u8::MAX)
        .filter(|i| idsdb.get_class(i).is_ok())
        .map(|i| idsdb.get_class(&i).unwrap())
        .for_each(|class| writeln!(o, "{} => Some(\"{}\"),", class.id, class.name).unwrap());
    writeln!(o, "_ => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    writeln!(
        o,
        "#[inline]\nconst fn classify_subclass(class: u8, subclass: u8) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(o, "match (class, subclass) {{").unwrap();
    for i in 0..u8::MAX {
        for j in 0..u8::MAX {
            if let Ok(class) = idsdb.get_class(&i) {
                if let Ok(subclass) = class.get_subclass(&j) {
                    writeln!(
                        o,
                        "({}, {}) => Some(\"{}\"),",
                        class.id, subclass.id, subclass.name
                    )
                    .unwrap();
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }
    }
    writeln!(o, "(_, _) => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    writeln!(o, "#[inline]\nconst fn classify_prog_if(class: u8, subclass: u8, interface: u8) -> Option<&'static str> {{").unwrap();
    writeln!(o, "match (class, subclass, interface) {{").unwrap();
    for i in 0..u8::MAX {
        for j in 0..u8::MAX {
            for k in 0..u8::MAX {
                if let Ok(class) = idsdb.get_class(&i) {
                    if let Ok(subclass) = class.get_subclass(&j) {
                        if let Ok(interface) = subclass.get_prog_interface(&k) {
                            writeln!(
                                o,
                                "({}, {}, {}) => Some(\"{}\"),",
                                class.id, subclass.id, interface.id, interface.name
                            )
                            .unwrap();
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        }
    }
    writeln!(o, "(_, _, _) => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    o.shrink_to_fit();
    gen_file_str("pciids.rs", o.as_str());
}
