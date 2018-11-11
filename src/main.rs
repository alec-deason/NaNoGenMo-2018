extern crate novel_gen;

fn main() {
    let mut w = novel_gen::World::new();
    while w.time < 360.0*24.0 {
        w.step_simulation();
        eprintln!("{}", w.time);
    }
    w.show_events(0);
}
