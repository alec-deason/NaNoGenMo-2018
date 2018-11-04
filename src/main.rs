extern crate novel_gen;

fn main() {
    let mut w = novel_gen::World::new();
    while w.time < 100.0 {
        w.step_simulation();
    }
    w.show_events(0);
}