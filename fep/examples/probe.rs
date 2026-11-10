use std::sync::Arc;
use fep::{Fep, Instance, Resident};
fn main() {
    let root = "D:/KH-Extract/ddd/original/effect";
    let frr = Arc::new(Resident::parse(std::fs::read(format!("{root}/resident/bin/resident.frr")).unwrap()).unwrap());
    for (file, name, parts) in [("sys/ffa/bin/p_fa_ajc0.fep", "P_FA_AJC_SHO0_0", vec![18usize, 19]), ("sys/ffa/bin/p_fa_com0.fep", "P_FA_COM_LOC0_0", vec![0, 1]), ("sys/ffa/bin/p_fa_com0.fep", "P_FA_COM_BSS0_0", vec![2, 12]), ("sys/ffa/bin/p_fa_ajc0.fep", "P_FA_AJC_CAL0_0", vec![0,1,2])] {
        let f = Arc::new(Fep::parse(std::fs::read(format!("{root}/{file}")).unwrap()).unwrap());
        let (_, _, b) = f.element(name).unwrap();
        for &k in &parts {
            let d = f.bodies[b].parts[k].as_ref().unwrap();
            println!("{name} part {k}: mtype {} life {} slots {:?}", d.matrix_type, d.lifetime, d.slots.iter().map(|s| (s.id, s.inputs.iter().map(|i| i.slot).collect::<Vec<_>>())).collect::<Vec<_>>());
            for e in &d.init.entries { let s = &d.slots[e.slot as usize]; if s.id == 14 { let p = d.init.params(e); println!("   lmatrix bytes {:?}", &f.data[p..p+4]); } }
        }
        let opts = fep::Options { resident: Some(frr.clone()), ..Default::default() };
        let mut e = Instance::start_with(f.clone(), name, 0.0, opts).unwrap();
        for t in 0..24 {
            for p in e.parts().filter(|p| parts.contains(&p.index())) {
                let loc = p.local().map(|m| m.w_axis.truncate());
                let mut extra = String::new();
                for d in p.draws() { if let Some(tl) = &d.tail { extra += &format!(" tail key {} V {} facing {} world {} blend {} p0 {:?} col0 {:?} sc0 {:.3} last {:?}", tl.anm_frame, tl.reached, tl.facing, tl.world, tl.blend, tl.points[0].position, tl.points[0].colour, tl.points[0].scale, tl.points.last().map(|q| (q.position, q.colour, q.scale))); } }
                let outs: Vec<_> = (0..p.lmatrix_flags() as usize*0 + 9).filter_map(|s| p.output(s)).collect();
                if name.contains("BSS0") { extra += &format!(" outs {:?}", outs); }
                println!("t {t} part {} id {} parent {:?} time {} local {:?}{extra}", p.index(), p.id(), p.parent(), p.time(), loc);
            }
            e.step(1.0);
            if e.is_ended() { break; }
        }
    }
}
