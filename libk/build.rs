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
    for class in (0..u8::MAX)
        .filter(|i| idsdb.get_class(i).is_ok())
        .map(|i| idsdb.get_class(&i).unwrap())
        .collect::<Vec<_>>()
    {
        writeln!(o, "{} => Some(\"{}\"),", class.id, class.name).unwrap();
    }
    writeln!(o, "_ => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    writeln!(
        o,
        "#[inline]\nconst fn classify_subclass(class: u8, subclass: u8) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(o, "match (class, subclass) {{").unwrap();
    for class in (0..u8::MAX)
        .filter(|i| idsdb.get_class(i).is_ok())
        .map(|i| idsdb.get_class(&i).unwrap())
        .collect::<Vec<_>>()
    {
        for subclass in (0..u8::MAX)
            .filter(|j| class.get_subclass(j).is_ok())
            .map(|j| class.get_subclass(&j).unwrap())
            .collect::<Vec<_>>()
        {
            writeln!(
                o,
                "({}, {}) => Some(\"{}\"),",
                class.id, subclass.id, subclass.name
            )
            .unwrap();
        }
    }
    writeln!(o, "(_, _) => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    writeln!(o, "#[inline]\nconst fn classify_prog_if(class: u8, subclass: u8, interface: u8) -> Option<&'static str> {{").unwrap();
    writeln!(o, "match (class, subclass, interface) {{").unwrap();
    for class in (0..u8::MAX)
        .filter(|i| idsdb.get_class(i).is_ok())
        .map(|i| idsdb.get_class(&i).unwrap())
        .collect::<Vec<_>>()
    {
        for subclass in (0..u8::MAX)
            .filter(|j| class.get_subclass(j).is_ok())
            .map(|j| class.get_subclass(&j).unwrap())
            .collect::<Vec<_>>()
        {
            for interface in (0..u8::MAX)
                .filter(|k| subclass.get_prog_interface(k).is_ok())
                .map(|k| subclass.get_prog_interface(&k).unwrap())
                .collect::<Vec<_>>()
            {
                writeln!(
                    o,
                    "({}, {}, {}) => Some(\"{}\"),",
                    class.id, subclass.id, interface.id, interface.name
                )
                .unwrap();
            }
        }
    }
    writeln!(o, "(_, _, _) => None,").unwrap();
    writeln!(o, "}}\n}}\n").unwrap();
    o.shrink_to_fit();
    gen_file_str("pciids.rs", o.as_str());
}
