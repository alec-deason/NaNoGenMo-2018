#![feature(vec_remove_item)]
#![feature(closure_to_fn_coercion)]
#![feature(fnbox)]
extern crate rand;

mod names;
mod mind;
mod conversation;

use std::mem;
use std::boxed::FnBox;
use rand::prelude::SliceRandom;
use std::fmt;
use rand::Rng;
use std::rc::Rc;
use rand::distributions::{Normal, Distribution};
use std::cell::{RefCell, Cell};

const STEP_SIZE: f64 = 1.0/(360.0*24.0);

pub struct World {
    pub time: f64,
    pub agent_time: f64,
    pub agents: Vec<Rc<RefCell<Agent>>>,
    living_agents: usize,
    pub locations: Vec<Rc<Location>>,
    pub items: Vec<Rc<Item>>,
}

impl World {
    pub fn new() -> World {
        let world_scale = 10;
        let mut rng = rand::thread_rng();

        let locations: Vec<Rc<Location>> = (0..(10*world_scale)).into_iter()
            .map(|id| Rc::new(Location::new(id)))
            .collect();

        let mut w = World {
            time: 0.0,
            agent_time: 0.0,
            agents: Vec::new(),
            living_agents: 0,
            locations: locations,
            items: Vec::new(),
        };

        let loc_count = w.locations.len();

        for loc in &w.locations {
            for _ in 0..rng.gen_range(1, 3) {
                let idx = rng.gen_range(0, loc_count);
                let exit = w.locations[idx].clone();
                loc.exits.borrow_mut().push(exit);
            }
        }

        for id in 0..world_scale {
            let idx = rng.gen_range(0, loc_count);
            let loc = w.locations[idx].clone();
            let a = Rc::new(RefCell::new(Agent::new(id, loc.clone())));
            loc.agents.borrow_mut().push(a.clone());
            w.agents.push(a);
            w.living_agents += 1;
        }

        for _ in 0..(5*world_scale) {
            let idx = rng.gen_range(0, loc_count);
            let loc = w.locations[idx].clone();
            let item = Rc::new(Item::new(w.items.len(), loc.clone()));
            w.items.push(item.clone());
            loc.items.borrow_mut().push(item);
        }

        w
    }

    pub fn step_simulation(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..self.living_agents/5 {
            let loc = self.locations.choose(&mut rng).unwrap();
            let item = Rc::new(Item::new(self.items.len(), loc.clone()));
            self.items.push(item.clone());
            loc.items.borrow_mut().push(item);
        }

        for agent_rc in &self.agents {
            let old_location;
            let new_location;
            {
                let mut agent = agent_rc.borrow_mut();
                if !agent.health.alive { continue; };
                old_location = agent.location.clone();
                old_location.agent_time.set(old_location.agent_time.get() +STEP_SIZE);
                self.agent_time += STEP_SIZE;
                agent.step_simulation(self);
                if !agent.health.alive {
                    self.living_agents -= 1;
                }
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

#[derive(Clone, PartialEq)]
pub struct Event {
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

pub struct Agent {
    pub id: usize,
    pub name: String,
    sex: Sex,
    age: f64,
    pub health: Health,
    pub mind: mind::Mind,
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
        let sex = *[Sex::Male, Sex::Female].choose(&mut rng).unwrap();
        //TODO Normal isn't the right distribution for this
        let age = match sex {
            Sex::Female => Normal::new(38.1, 10.0).sample(&mut rng).min(120.0).max(0.0),
            Sex::Male => Normal::new(36.8, 10.0).sample(&mut rng).min(120.0).max(0.0),
        };
        let first_name = match sex {
            Sex::Male => names::MALE_FIRST_NAMES.choose(&mut rng).unwrap(),
            Sex::Female => names::FEMALE_FIRST_NAMES.choose(&mut rng).unwrap(),
        };
        let surname = names::SURNAMES.choose(&mut rng).unwrap();

        let full_name = format!("{} {}", first_name, surname).to_string();

        let mut a = Agent {
            id: id,
            name: full_name,
            sex: sex,
            age: age,
            health: Health::new(),
            mind: mind::Mind::new(),
            location: location,
            events: Vec::new(),
            action_points: 0,
        };

        a.events.push(Event { msg: "Agent created".to_string() });

        a
    }

    fn step_simulation(&mut self, world: &World) {
        let mut rng = rand::thread_rng();
        let view = HealthView { mind: &self.mind, location: self.location.clone(), events: &mut self.events };
        self.health.step_simulation(view);

        if !self.health.alive {
            self.events.push(Event { msg: "Died".to_string() });
            return;
        }

        let view = mind::MindView { health: &self.health, location: self.location.clone(), events: &mut self.events };
        self.mind.step_simulation(view);

        self.age += STEP_SIZE;

        let mut potential_actions = Vec::with_capacity(30);
        potential_actions.extend(self.health.actions(self));
        potential_actions.extend(self.mind.actions(self));

        match potential_actions.choose_weighted_mut(&mut rng, |(w, _)| *w) {
            Ok((_, actual_action)) => {
                actual_action(self);
            },
            Err(_) => (),
        };
    }
}

struct HealthView<'a> {
    mind: &'a mind::Mind,
    location: Rc<Location>,
    events: &'a mut Vec<Event>,
}

pub struct Health {
    chronic: f64,
    acute: f64,
    pain: f64,
    sleepiness: f64,
    hunger: f64,
    alive: bool,
    awake: bool,

}

impl ActionSource  for Health {
    fn actions(& self, gent: &Agent) -> Vec<(f64, fn(&mut Agent))> {
        let mut actions:Vec<(f64, fn(&mut Agent))> = Vec::new();

        if self.sleepiness > 16.0 && self.awake {
            actions.push((1.0, |agent| {
                agent.health.awake = false;
                agent.events.push(Event { msg: "Went to sleep".to_string() })
            }));
        } else if self.sleepiness <= 0.0 && !self.awake {
            actions.push((1.0, |agent| {
                agent.health.awake = true;
                agent.events.push(Event { msg: "Woke up".to_string() })
            }));
        }
        if self.awake && self.hunger > 0.3 {
            actions.push((1.0, |agent| {
                let mut food = Vec::new();
                for i in agent.location.items.borrow().iter() {
                    if i.food_value > 0.0 {
                        food.push(i.clone());
                    }
                }

                let mut rng = rand::thread_rng();
                let thing_to_eat = food.choose(&mut rng);

                match thing_to_eat {
                    Some(thing_to_eat) => {
                        agent.health.hunger = (agent.health.hunger - thing_to_eat.food_value).max(0.0);
                        agent.location.items.borrow_mut().remove_item(thing_to_eat);
                        agent.events.push(Event { msg: format!("Ate {}", thing_to_eat.name).to_string() });
                        agent.mind.cheer = (agent.mind.cheer + 0.5).min(1.0);
                    },
                    None => agent.mind.cheer = (agent.mind.cheer - 0.05).max(-1.0), // Nothing to eat :(
                };
            }));
        }
        actions
    }
}

impl Health {
    fn new() -> Health {
        let mut rng = rand::thread_rng();
        Health {
            chronic: Normal::new(0.75, 0.01).sample(&mut rng).min(1.0).max(0.0),
            acute: 1.0,
            pain: 0.0,
            sleepiness: 0.0,
            hunger: Normal::new(0.5, 0.01).sample(&mut rng).min(1.0).max(0.0),
            alive: true,
            awake: true,
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

        if self.awake {
            self.sleepiness += 1.0;
        } else {
            self.sleepiness -= 1.0;
        }

        if self.sleepiness > 20.0 {
            self.pain += 0.01;
        }

        self.hunger += 0.01;
        self.acute += 0.001;

        self.chronic = self.chronic.min(1.0).max(0.0);
        self.acute = self.acute.min(1.0).max(0.0);
        self.pain = self.pain.min(1.0).max(0.0);
        self.hunger = self.hunger.min(1.0).max(0.0);

    }
}
pub struct Location {
    pub id: usize,
    pub exits: RefCell<Vec<Rc<Location>>>,
    agents: RefCell<Vec<Rc<RefCell<Agent>>>>,
    items: RefCell<Vec<Rc<Item>>>,
    pub agent_time: Cell<f64>,
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
            items: RefCell::new(Vec::new()),
            agent_time: Cell::new(0.0),
        }
    }
}

pub struct Item {
    pub id: usize,
    name: String,
    food_value: f64,
    monetary_value: f64,
    location: Rc<Location>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self as *const _ == other as *const _
    }
}

impl Item {
    fn new(id: usize, location: Rc<Location>) -> Item {
        Item {
            id: id,
            name: "Food".to_string(),
            food_value: 1.0,
            monetary_value: 0.0,
            location: location,
        }
    }
}

pub struct PreActionView<'a> {
    pub mind: &'a mind::Mind,
    pub health: &'a Health,
    pub location: Rc<Location>,
}

pub struct ActionView<'a> {
    pub mind: &'a mut mind::Mind,
    pub health: &'a Health,
    pub location: Rc<Location>,
    pub events: &'a mut Vec<Event>,
}

pub trait ActionSource {
    fn actions(&self, agent: &Agent) -> Vec<(f64, fn(&mut Agent))>;
}
