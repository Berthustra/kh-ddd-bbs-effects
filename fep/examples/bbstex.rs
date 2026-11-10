fn main() {
    for e in std::fs::read_dir("D:/KH-Extract/bbs/arc/effect").unwrap().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !e.path().is_dir() || !(name.starts_with("pst_") || name.starts_with("p0") || name.starts_with("p1")) { continue; }
        for f in std::fs::read_dir(e.path()).unwrap().flatten() {
            let p = f.path();
            if p.extension().is_some_and(|x| x == "fep") {
                let fp = fep::Fep::parse(std::fs::read(&p).unwrap()).unwrap();
                let own: Vec<_> = fp.tex.iter().filter(|r| !r.is_resident() && r.id != 0).map(|r| format!("{:#x}", r.id)).collect();
                println!("{name}: {} tex, own {:?}", fp.tex.len(), own);
            }
        }
    }
}
