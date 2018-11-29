mod events;
mod executive;
mod daemons;

use rand::seq::SliceRandom;
use rand::seq::IteratorRandom;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::{Event, Item, ItemId, World, LocationId};

pub type AgentId = usize;
pub struct Agent {
    id: AgentId,
    cheer: f64,
    pub location: usize,
    pub events: Vec<Box<dyn Event>>,
    inventory: HashMap<ItemId, Item>,

    health: RefCell<Health>,
    mind: RefCell<Mind>,

    daemons: Vec<Box<dyn daemons::Daemon>>,
}

impl Agent {
    pub fn new(id: AgentId) -> Agent {
        Agent {
            id: id,
            cheer: 0.0,
            location: 0,
            events: Vec::with_capacity(1000),
            inventory: HashMap::with_capacity(10),

            health: RefCell::new(Health::new()),
            mind: RefCell::new(Mind::new()),
            
            daemons: vec![
//                Box::new(Wanderlust { last_wander: Cell::new(0.0) }),
                Box::new(daemons::HungerTracker {}),
                Box::new(daemons::SleepTracker {}),
                Box::new(daemons::PoopTracker {}),
                Box::new(executive::Executive {}),
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


struct Mind {
    goals: HashMap<executive::Goal, f64>,
    current_goal: Option<(executive::Goal, Box<dyn executive::Strategy>)>,
    agitation: f64,
    cheer: f64,
}

impl Mind {
    fn new() -> Mind {
        Mind {
            goals: HashMap::with_capacity(100),
            current_goal: None,
            agitation: 0.0,
            cheer: 1.0,
        }
    }
}
