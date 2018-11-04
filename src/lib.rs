#![feature(vec_remove_item)]
extern crate rand;

mod names;
use rand::Rng;
use rand::distributions::{Normal, Distribution};

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

#[derive(Copy, Clone)]
enum Sex {
    Male,
    Female,
}

struct Health {
    chronic: f64,
    acute: f64,
    pain: f64,
    hunger: f64,
    alive: bool,
}

impl Health {
    fn new() -> Health {
        let mut rng = rand::thread_rng();
        Health {
            chronic: Normal::new(0.75, 0.01).sample(&mut rng).min(1.0).max(0.0),
            acute: 1.0,
            pain: 0.0,
            hunger: Normal::new(0.5, 0.01).sample(&mut rng).min(1.0).max(0.0),
            alive: true,
        }
    }

    fn step(&mut self) {
        if self.hunger == 1.0 {
            self.acute -= 0.01;
        }

        if self.acute < 1.0 {
            self.pain += 0.01;
        } else {
            self.pain -= 0.1;
        }
        
        if self.acute < 0.2 {
            self.chronic -= 0.01;
        }

        if self.acute <= 0.0 || self.chronic <= 0.0 {
            self.alive = false;
        }

        self.hunger += 0.01;
        self.acute += 0.001;
    
        self.chronic = self.chronic.min(1.0).max(0.0);
        self.acute = self.acute.min(1.0).max(0.0);
        self.pain = self.pain.min(1.0).max(0.0);
        self.hunger = self.hunger.min(1.0).max(0.0);
    }
}

pub struct Agent {
    pub name: String,
    events: Vec<Event>,
    age: f64,
    sex: Sex,
    location: usize,
    health: Health,

    action_points: i32,
}

impl Agent {
    pub fn new() -> Agent {
        let mut rng = rand::thread_rng();
        let sex = *rng.choose(&[Sex::Male, Sex::Female]).unwrap();
        let age = rng.gen_range(0.0, 120.0);
        let first_name = match sex {
            Sex::Male => rng.choose(names::MALE_FIRST_NAMES).unwrap(),
            Sex::Female => rng.choose(names::FEMALE_FIRST_NAMES).unwrap(),
        };
        let surname = rng.choose(names::SURNAMES).unwrap();

        let full_name = format!("{} {}", first_name, surname).to_string();

        Agent { events: Vec::new(), name: full_name, age: age, sex: sex, location: 0, health: Health::new(), action_points: 0, }
    }
}

pub struct World {
    pub agents: Vec<Agent>,
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
            println!("{}: {} {}", event.time, self.agents[event.agent].name, event.desc);
        }
    }

    pub fn step_simulation(&mut self) {
        let mut index: Vec<usize> = (0..self.agents.len()).collect();
        self.rng.shuffle(&mut index);
        for i in index {
            let a = &mut self.agents[i];
            if a.health.alive {
                a.health.step();
                if !a.health.alive {
                    a.events.push( Event { location: a.location,
                                           agent: i,
                                           time: self.time,
                                           desc: "Died".to_string(),
                    } );
                    break
                }
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
        }
        self.time += 1.0;
    }
}
