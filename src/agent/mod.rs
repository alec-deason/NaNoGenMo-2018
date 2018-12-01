mod events;
mod names;
mod executive;
mod daemons;

use rand::seq::SliceRandom;
use rand::seq::IteratorRandom;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use super::{Event, Item, ItemId, World, LocationId};

pub type AgentId = usize;
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub location: usize,
    pub events: Vec<Box<dyn Event>>,
    inventory: HashMap<ItemId, Item>,

    pub health: RefCell<Health>,
    mind: RefCell<Mind>,

    pub total_time: Cell<f64>,

    daemons: Vec<Box<dyn daemons::Daemon>>,
}

impl Agent {
    pub fn new(id: AgentId) -> Agent {
        Agent {
            id: id,
            name: names::male_name(),
            total_time: Cell::new(0.0),
            location: 0,
            events: Vec::with_capacity(1000),
            inventory: HashMap::with_capacity(10),

            health: RefCell::new(Health::new()),
            mind: RefCell::new(Mind::new()),
            
            daemons: vec![
                Box::new(daemons::Wanderlust { last_wander: Cell::new(0.0) }),
                Box::new(daemons::HungerTracker {}),
                Box::new(daemons::SleepTracker {}),
                Box::new(daemons::PoopTracker {}),
                Box::new(daemons::PainTracker {}),
                Box::new(daemons::EncounterTracker::new()),
                Box::new(executive::Executive {}),
            ],
        }
    }

    pub fn step_simulation(&self, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        let mut daemon_urgency: Vec<f64> = Vec::with_capacity(self.daemons.len());
        let mut potential_daemons = Vec::with_capacity(self.daemons.len());

        self.total_time.set(self.total_time.get() + 1.0);

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

pub struct Health {
    pub alive: bool,
    awake: bool,
    hunger: f64,
    sleepiness: f64,
    poop: f64,
    pain: f64,
}

impl Health {
    fn new() -> Health {
        Health {
            alive: true,
            awake: true,
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
    paused_goals: Vec<(executive::Goal, Box<dyn executive::Strategy>)>,
    opinions_on_others: HashMap<AgentId, f64>,
    opinions_on_places: HashMap<LocationId, f64>,
    agitation: f64,
    cheer: f64,
}

impl Mind {
    fn new() -> Mind {
        Mind {
            goals: HashMap::with_capacity(100),
            current_goal: None,
            paused_goals: Vec::with_capacity(5),
            opinions_on_others: HashMap::with_capacity(100),
            opinions_on_places: HashMap::with_capacity(100),
            agitation: 0.0,
            cheer: 1.0,
        }
    }
}
