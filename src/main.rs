extern crate novel_gen;

use std::time::SystemTime;

use std::fs::File;

//mod social_graph;
mod location_graph;

fn main() {
    let start = SystemTime::now();

    let mut w = novel_gen::World::new(10);
    while w.time < 0.5*360.0*24.0 {
        w.step_simulation();
        eprintln!("{}", w.time);
    }

    let total_secs = start.elapsed().unwrap().as_secs() as f64;
    let total_secs = total_secs + start.elapsed().unwrap().subsec_millis() as f64 / 1000.0;
    let agent_time = w.agents.iter().map(|a| a.total_time.get() ).fold(0.0, |acc, x| acc + x);;
    let agent_days = agent_time / 24.0;

    let agent_idx = w.agents.iter().fold((0usize, 0.0), |acc, a| {
        let tt = a.total_time.get();
        if tt > acc.1 {
            (a.id, tt)
        } else { acc }
    }).0;
    w.show_events(agent_idx);
    println!("Total Agent days: {}", agent_days);
    println!("This Agent days: {}", w.agents[agent_idx].total_time.get() / 24.0);
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
