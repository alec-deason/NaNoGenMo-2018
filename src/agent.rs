use rand::seq::SliceRandom;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::{Event, DummyEvent, Item, World, LocationId};

pub type AgentId = usize;
pub struct Agent {
    id: AgentId,
    cheer: f64,
    pub location: usize,
    pub events: Vec<Box<dyn Event>>,
    inventory: Vec<Item>,

    health: RefCell<Health>,
    mind: RefCell<Mind>,

    daemons: Vec<Box<dyn Daemon>>,
}

impl Agent {
    pub fn new(id: AgentId) -> Agent {
        Agent {
            id: id,
            cheer: 0.0,
            location: 0,
            events: Vec::new(),
            inventory: Vec::new(),

            health: RefCell::new(Health::new()),
            mind: RefCell::new(Mind::new()),
            
            daemons: vec![
                Box::new(Wanderlust { last_wander: Cell::new(0.0) }),
                Box::new(HungerTracker {}),
                Box::new(SleepTracker {}),
                Box::new(PoopTracker {}),
            ],
        }
    }

    pub fn step_simulation(&self, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        let mut daemon_urgency: Vec<f64> = Vec::with_capacity(self.daemons.len());
        let mut potential_daemons = Vec::with_capacity(self.daemons.len());

        for daemon in &self.daemons {
            match daemon.step_simulation(self, world) {
                Some(urgency) => {
                    daemon_urgency.push(urgency);
                    potential_daemons.push(daemon);
                },
                None => (),
            }
        }

        let choices: Vec<(usize, &f64)> = daemon_urgency.iter().enumerate().collect();
        match choices.choose_weighted(&mut rng, |k| k.1) {
            Ok((i, _)) => {
                potential_daemons[*i].events(self, world)
            },
            Err(_) => vec![],
        }
    }
}

#[derive(Copy, Clone)]
struct MoveEvent {
    start: LocationId,
    end: LocationId,
    agent: AgentId,
}

impl Event for MoveEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];

        world.locations[self.start].agents.remove_item(&self.agent);
        world.locations[self.end].agents.push(self.agent);
        agent.location = self.end;
        agent.events.push(Box::new(*self));
    }

    fn to_string(&self, world: &World) -> String {
        let start = &world.locations[self.start].name;
        let end = &world.locations[self.start].name;

        format!("Moved from {} to {}.", start, end).to_string()
    }
}

struct PickupEvent {
    location: LocationId,
    item: LocationId,
    agent: AgentId,
}

impl Event for PickupEvent {
    fn apply(&self, world: &mut World) {
        let location = &mut world.locations[self.location];
        let agent = &mut world.agents[self.agent];

        match location.items.remove_entry(&self.item) {
            Some((_, item)) => {
                agent.events.push(Box::new(DummyEvent {
                    agent: agent.id,
                    message: format!("Picked up {}.", item.name).to_string(),
                }));
                agent.inventory.push(item);
            },
            None => {
                agent.events.push(Box::new(DummyEvent {
                    agent: agent.id,
                    message: "Tried to pick something up but it wasn't there".to_string(),
                }));
            }
        }
    }

    fn to_string(&self, world: &World) -> String {
        "Trying to pick something up.".to_string()
    }
}

trait Daemon {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64>;
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![]
    }
}

struct Wanderlust {
    last_wander: Cell<f64>,
}


impl Daemon for Wanderlust {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64> {
        let min_wait = 5.0;
        let max_wait = 50.0;
        let wait = world.time - self.last_wander.get();
        
        if wait > min_wait {
            Some((wait / max_wait).min(1.0))
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        self.last_wander.set(world.time);
        let new_loc = *world.locations[agent.location].exits.choose(&mut rng).unwrap_or(&agent.location);
        if new_loc != agent.location {
            vec![
                Box::new(MoveEvent { start: agent.location, end: new_loc, agent: agent.id }),
            ]
        } else {
            vec![]
        }
    }
}

struct HungerTracker;
impl Daemon for HungerTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.hunger += 0.1;

        if health.hunger > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(Goal::FindFood).or_insert(0.0);
            *goal += 0.5;
        }

        if health.hunger > 10.0 {
            health.pain += 0.1;
        }

        None
    }
}

#[derive(Copy, Clone)]
struct NapEvent {
    agent: AgentId,
}
impl Event for NapEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));
        let mut health = agent.health.borrow_mut();
        health.sleepiness = 0.0;
    }
    fn to_string(&self, world: &World) -> String {
        format!("Took a nap.").to_string()
    }
}

struct SleepTracker;
impl Daemon for SleepTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.sleepiness += 0.1;

        if health.sleepiness > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(Goal::Rest).or_insert(0.0);
            *goal += 0.5;
        }

        if health.sleepiness > 10.0 {
            Some(10.0)
        } else {
            None
        }
    }
    
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(NapEvent { agent: agent.id })
        ]
    }
}

#[derive(Copy, Clone)]
struct DefecateEvent {
    agent: AgentId,
}
impl Event for DefecateEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));
        let mut health = agent.health.borrow_mut();
        health.poop = 0.0;
        *world.metrics.entry("shit").or_insert(0) += 1;
    }
    fn to_string(&self, _: &World) -> String {
        format!("Took a shit.").to_string()
    }
}

struct PoopTracker;
impl Daemon for PoopTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.poop += 0.1;

        if health.poop > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(Goal::Shit).or_insert(0.0);
            *goal += 0.5;
        }

        if health.poop > 2.0 {
            health.pain += 0.1;
        }

        if health.poop > 10.0 {
            Some(10.0)
        } else {
            None
        }
    }
    
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(DefecateEvent { agent: agent.id })
        ]
    }
}

struct Health {
    hunger: f64,
    sleepiness: f64,
    poop: f64,
    pain: f64,
}

impl Health {
    fn new() -> Health {
        Health {
            hunger: 0.0,
            pain: 0.0,
            poop: 0.0,
            sleepiness: 0.0,
        }
    }
}

#[derive(Hash, PartialEq)]
enum Goal {
    FindFood,
    Rest,
    Shit,
}
impl Eq for Goal {}

struct Mind {
    goals: HashMap<Goal, f64>,
    agitation: f64,
    cheer: f64,
}

impl Mind {
    fn new() -> Mind {
        Mind {
            goals: HashMap::with_capacity(100),
            agitation: 0.0,
            cheer: 1.0,
        }
    }
}
