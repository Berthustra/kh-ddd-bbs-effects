//! Lists each FEP file's element names and whether each starts (fep::Instance::start_with).
fn main() {
    let dir = r"D:\KH-Extract\ddd\original";
    for p in std::env::args().skip(1) {
        let d = std::fs::read(format!(r"{dir}\{p}")).unwrap();
        let f = std::sync::Arc::new(fep::Fep::parse(d).unwrap());
        println!("== {p}");
        for n in &f.names {
            match fep::Instance::start_with(f.clone(), &n.name, 0.0, Default::default()) {
                Ok(mut e) => {
                    let looping = e.fep().bodies[e.body_index()].flags & 2 != 0;
                    let mut k = 0;
                    while !e.is_ended() && k < 2000 { e.step(0.5); k += 1; }
                    println!("ok  {} loop={} frames={}", n.name, looping, k as f32 * 0.5);
                }
                Err(err) => println!("ERR {} {err}", n.name),
            }
        }
    }
}
