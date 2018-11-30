extern crate novel_gen;

use std::time::SystemTime;

use std::fs::File;

//mod social_graph;
mod location_graph;

fn main() {
    let start = SystemTime::now();

    let mut w = novel_gen::World::new(6);
    while w.time < 180.0*24.0 {
        w.step_simulation();
        eprintln!("{}", w.time);
    }
    w.show_events(0);

    let total_secs = start.elapsed().unwrap().as_secs() as f64;
    let total_secs = total_secs + start.elapsed().unwrap().subsec_millis() as f64 / 1000.0;
    let agent_days = (w.time / 24.0) * w.agents.len() as f64;
    println!("Agent days: {}", agent_days);
    println!("Real seconds: {}", total_secs);
    println!("Agent days per second: {}", agent_days / total_secs);
    for (name, count) in &w.metrics {
        let per_sec = *count as f64 / total_secs;
        println!("{:.2} {}s per second", per_sec, name);
    }
   //let mut f = File::create("social.dot").unwrap();
   // social_graph::render_to(&w.agents, &mut f);
   let mut f = File::create("location.dot").unwrap();
   location_graph::render_to(&w.locations, &w, &mut f);
}
