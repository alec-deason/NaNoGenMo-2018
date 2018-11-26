#![feature(vec_remove_item)]

mod agent;

use rand::{Rng};
use rand::seq::IteratorRandom;
use std::collections::HashMap;

use self::agent::{Agent, AgentId};

pub struct World {
    pub time: f64,
    agents: Vec<Agent>,
    locations: Vec<Location>,
}

type ItemId = usize;
struct Item {
    id: ItemId,
    name: String,
    food_value: f64,
}

type LocationId = usize;
struct Location {
    id: LocationId,
    name: String,
    agents: Vec<AgentId>,
    items: HashMap<ItemId, Item>,
    exits: Vec<LocationId>,
}

trait Event {
    fn apply(&self, world: &mut World);
    fn to_string(&self, world: &World) -> String;
}



#[derive(Clone)]
struct DummyEvent {
    agent: AgentId,
    message: String,
}
impl Event for DummyEvent {
    fn apply(&self, world: &mut World) {
        world.agents[self.agent].events.push(Box::new(self.clone()));
    }

    fn to_string(&self, _: &World) -> String {
        self.message.clone()
    }
}

impl World {
    pub fn new() -> World {
        let scale:i32 = 10;
        let bushyness:i32 = 10;
        let location_count:i32 = scale*10;
        let agent_count:i32 = scale;

        let mut rng = rand::thread_rng();

        let mut w = World {
            time: 0.0,
            agents: Vec::new(),
            locations: Vec::new(),
        };

        for id in 0..location_count {
            let exit_count = rng.gen_range(1, bushyness) as usize;
            let exits = (0..location_count as LocationId).choose_multiple(&mut rng, exit_count);

            let mut location = Location::new(id as LocationId);
            location.exits.extend(exits);

            w.locations.push(location);
        }
        
        for id in 0..agent_count {
            let mut a = Agent::new(id as AgentId);
            a.location = (0..w.locations.len()).choose(&mut rng).unwrap_or(0) as LocationId;
            w.agents.push(a);
        }

        w
    }

    pub fn step_simulation(&mut self) {
        self.time += 1.0;
        let mut events = Vec::with_capacity(self.agents.len());
        for a in &self.agents {
            events.extend(a.step_simulation(self));
        }
        for event in events {
            event.apply(self)
        }
    }

    pub fn show_events(&self, agent_id: AgentId) {
        let a = &self.agents[agent_id];
        for event in &a.events {
            println!("{}", event.to_string(self));
        }
    }
}

impl Location {
    fn new(id: LocationId) -> Location {
        Location {
            id: id,
            name: "a place".to_string(),
            agents: Vec::with_capacity(10),
            items: HashMap::with_capacity(10),
            exits: Vec::with_capacity(10),
        }
    }
}
