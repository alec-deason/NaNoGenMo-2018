extern crate novel_gen;

use std::fs::File;

mod social_graph;

fn main() {
    let mut w = novel_gen::World::new();
    while w.time < 360.0*24.0 {
        w.step_simulation();
        eprintln!("{}", w.time);
    }
    w.show_events(0);
    let mut f = File::create("example1.dot").unwrap();
    social_graph::render_to(w.agents, &mut f);
}
