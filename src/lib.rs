#![feature(vec_remove_item)]
extern crate rand;

mod names;

use std::fmt;
use rand::Rng;
use std::rc::Rc;
use rand::distributions::{Normal, Distribution};
use std::cell::RefCell;

const STEP_SIZE: f64 = 1.0/(360.0*24.0);

pub struct World {
    pub time: f64,
    agents: Vec<Rc<RefCell<Agent>>>,
    locations: Vec<Rc<Location>>,
}

impl World {
    pub fn new() -> World {
        let mut rng = rand::thread_rng();

        let locations: Vec<Rc<Location>> = (0..10000).into_iter()
            .map(|id| Rc::new(Location::new(id)))
            .collect();

        let mut w = World {
            time: 0.0,
            agents: Vec::new(),
            locations: locations,
        };

        let loc_count = w.locations.len();

        for loc in &w.locations {
            for _ in 0..rng.gen_range(1, 10) {
                let idx = rng.gen_range(0, loc_count);
                let exit = w.locations[idx].clone();
                loc.exits.borrow_mut().push(exit);
            }
        }

        for id in 0..1000 {
            let idx = rng.gen_range(0, loc_count);
            let loc = w.locations[idx].clone();
            let a = Rc::new(RefCell::new(Agent::new(id, loc.clone())));
            loc.agents.borrow_mut().push(a.clone());
            w.agents.push(a.clone());
        }

        w
    }

    pub fn step_simulation(&mut self) {
        for agent_rc in &self.agents {
            let old_location;
            let new_location;
            {
                let mut agent = agent_rc.borrow_mut();
                if !agent.health.alive { continue; };
                old_location = agent.location.clone();
                agent.step_simulation();
                new_location = agent.location.clone();
            }

            if old_location != new_location {
                old_location.agents.borrow_mut().remove_item(agent_rc);
                new_location.agents.borrow_mut().push(agent_rc.clone());
            }
        }
        self.time += 1.0;
    }

    pub fn show_events(&self, agent_index: usize) {
        let a = self.agents[agent_index].borrow();
        println!("Events for: {}", a);
        for event in &a.events {
            println!("{}", event.msg);
        }
    }
}

struct Event {
    msg: String,
}

#[derive(Copy, Clone, PartialEq)]
enum Sex {
    Male,
    Female,
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Sex::Male => write!(f, "Male"),
            Sex::Female => write!(f, "Female"),
        }
    }
}

struct Agent {
    id: usize,
    name: String,
    sex: Sex,
    age: f64,
    health: Health,
    mind: Mind,
    location: Rc<Location>,
    events: Vec<Event>,

    action_points: u32,
}

impl PartialEq for Agent {
    fn eq(&self, other: &Agent) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.name, self.sex, self.age)
    }
}

impl Agent {
    fn new(id: usize, location: Rc<Location>) -> Agent {
        let mut rng = rand::thread_rng();
        let sex = *rng.choose(&[Sex::Male, Sex::Female]).unwrap();
        //TODO Normal isn't the right distribution for this
        let age = match sex {
            Sex::Female => Normal::new(38.1, 10.0).sample(&mut rng).min(120.0).max(0.0),
            Sex::Male => Normal::new(36.8, 10.0).sample(&mut rng).min(120.0).max(0.0),
        };
        let first_name = match sex {
            Sex::Male => rng.choose(names::MALE_FIRST_NAMES).unwrap(),
            Sex::Female => rng.choose(names::FEMALE_FIRST_NAMES).unwrap(),
        };
        let surname = rng.choose(names::SURNAMES).unwrap();

        let full_name = format!("{} {}", first_name, surname).to_string();

        let mut a = Agent {
            id: id,
            name: full_name,
            sex: sex,
            age: age,
            health: Health::new(),
            mind: Mind::new(),
            location: location,
            events: Vec::new(),
            action_points: 0,
        };

        a.events.push(Event { msg: "Agent created".to_string() });

        a
    }

    fn step_simulation(&mut self) {
        let view = HealthView { mind: &self.mind, location: self.location.clone() };
        self.health.step_simulation(view);

        if !self.health.alive {
            self.events.push(Event { msg: "Died".to_string() });
            return;
        }

        let view = MindView { health: &self.health, location: self.location.clone() };
        self.mind.step_simulation(view);

        self.age += STEP_SIZE;

        self.action_points += 1;

        if self.action_points >= 5 {
            self.action_points -= 5;
            let mut rng = rand::thread_rng();
            let loc = self.location.clone();
            let exits = loc.exits.borrow();
            let idx = rng.gen_range(0, exits.len());
            let new_location = exits[idx].clone();
            self.events.push(Event { msg: format!("Moved from {} to {}", self.location.id, new_location.id).to_string() });
            self.location = new_location;
        }
    }
}

struct HealthView<'a> {
    mind: &'a Mind,
    location: Rc<Location>,
}

struct MindView<'a> {
    health: &'a Health,
    location: Rc<Location>,
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

    fn step_simulation(&mut self, view: HealthView) {
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

struct Mind {
    encounters: Vec<u32>,
}

impl Mind {
    fn new() -> Mind {
        Mind {
            encounters: Vec::new(),
        }
    }

    fn step_simulation(&mut self, view: MindView) {
        for a in view.location.agents.borrow().iter() {
            match a.try_borrow() {
                Ok(a) => {
                    while self.encounters.len() <= a.id { self.encounters.push(0) }
                    self.encounters[a.id] += 1;
                },
                Err(_) => () // This is the current agent, fine.
            }
        }
    }
}

struct Location {
    id: usize,
    exits: RefCell<Vec<Rc<Location>>>,
    agents: RefCell<Vec<Rc<RefCell<Agent>>>>,
}


impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.id == other.id
    }
}
impl Location {
    fn new(id: usize) -> Location {
        Location {
            id: id,
            exits: RefCell::new(Vec::new()),
            agents: RefCell::new(Vec::new()),
        }
    }
}
