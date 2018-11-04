#![feature(vec_remove_item)]
extern crate rand;

use rand::Rng;

pub struct Event {
    agent: usize,
    location: usize,
    time: f64,
    desc: String,
}


pub struct Location {
    agents: Vec<usize>,
    capacity: usize,
    name: &'static str,
    exits: Vec<usize>,
}

impl Location {
    pub fn new() -> Location {
        Location { agents: Vec::new(), capacity: 10, name: &"The Park", exits: Vec::new(), }
    }
}

pub struct Agent {
    name: &'static str,
    events: Vec<Event>,
    age: f64,
    location: usize,

    action_points: i32,
}

impl Agent {
    pub fn new() -> Agent {
        Agent { events: Vec::new(), name: &"Joe Shmo", age: rand::thread_rng().gen_range(0.0, 120.0), location: 0, action_points: 0, }
    }
}

pub struct World {
    agents: Vec<Agent>,
    locations: Vec<Location>,
    pub time: f64,

    rng: rand::ThreadRng,
}

impl World {
    pub fn new() -> World {
        let mut w = World {
            agents: Vec::new(),
            locations: Vec::new(),
            time: 0.0,

            rng: rand::thread_rng(),
        };
        w.initialize_locations();
        w.initialize_agents();
        w
    }

    fn initialize_locations(&mut self) {
        for _ in 0..1000 {
            self.locations.push( Location::new() );
        }

        for _ in 0..5000 {
            let a = self.rng.gen_range(0, self.locations.len());
            let b = self.rng.gen_range(0, self.locations.len());
            self.locations[a].exits.push(b);
        }
    }

    fn initialize_agents(&mut self) {
        for _ in 0..1000 {
            let mut place: usize;
            while {place = self.rng.gen_range(0, self.locations.len()); self.locations[place].capacity <= self.locations[place].agents.len()} {}
            self.locations[place].agents.push(self.agents.len());
            let mut a = Agent::new();
            a.location = place;
            a.events.push( Event { location: place, agent: self.agents.len(), time: self.time, desc: "Agent created".to_string() } );
            self.agents.push( a );
        }
    }

    pub fn show_events(&self, agent: usize) {
        for event in &self.agents[agent].events {
            println!("{}: ({}) {}", event.time, self.locations[event.location].name, event.desc);
        }
    }

    pub fn step_simulation(&mut self) {
        let mut index: Vec<usize> = (0..self.agents.len()).collect();
        self.rng.shuffle(&mut index);
        for i in index {
            let a = &mut self.agents[i];
            a.age += 1.0;
            a.action_points += 1;
            if a.action_points > 5 {
                let new_location;
                {
                    let place = &self.locations[a.location];
                    if place.exits.is_empty() { continue; }
                    let exit = self.rng.gen_range(0, place.exits.len());
                    new_location = self.locations[a.location].exits[exit];
                }
                self.locations[a.location].agents.remove_item(&i);
                self.locations[new_location].agents.push(i);
                a.events.push( Event { location: a.location,
                                       agent: i,
                                       time: self.time,
                                       desc: format!("Moved from {} to {}", a.location, new_location),
                } );
                a.location = new_location;
                a.action_points -= 5;
            }
        }
        self.time += 1.0;
    }
}
