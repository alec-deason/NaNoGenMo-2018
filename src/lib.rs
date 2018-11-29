#![feature(vec_remove_item)]

mod agent;

use rand::{Rng};
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use std::collections::HashMap;

use self::agent::{Agent, AgentId};

pub struct World {
    pub time: f64,
    pub agents: Vec<Agent>,
    locations: Vec<Location>,
    pub metrics: HashMap<&'static str, i32>,
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
    fn apply(&self, world: &mut World) { }
    fn to_string(&self, world: &World) -> String { "".to_string() }
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
    pub fn new(scale:i32) -> World {
        let location_count:i32 = scale*20;
        let agent_count:i32 = scale;

        let mut rng = rand::thread_rng();

        let mut w = World {
            time: 0.0,
            agents: Vec::with_capacity(agent_count as usize),
            locations: Vec::with_capacity(location_count as usize),
            metrics: HashMap::new(),
        };

        let mut item_id = 0;

        let (locations, agents) = make_locations(location_count, agent_count);
        w.locations.extend(locations);
        w.agents.extend(agents);

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

fn make_locations(location_count: i32, agent_count: i32) -> (Vec<Location>, Vec<Agent>) {
    let mut rng = rand::thread_rng();
    let mut locations = Vec::with_capacity(location_count as usize);
    let mut item_id = 0 as ItemId;

    let mut the_greenwood = Vec::with_capacity(location_count as usize);
    let village_count = rng.gen_range(1, 5);
    let mut villages:Vec<Vec<usize>> = Vec::with_capacity(village_count);
    for _ in 0..village_count {
        villages.push(Vec::with_capacity((location_count as f64 / village_count as f64) as usize));
    }

    let greenwood_count = location_count as usize - village_count*10;

    while the_greenwood.len() < greenwood_count {
        let id = locations.len();
        let mut location = Location::new(id);
        for _ in 0..rng.gen_range(0, 15) {
            location.items.insert(item_id, Item {
                id: item_id,
                name: "berry".to_string(),
                food_value: 1.0,
            });
            item_id += 1;
        }
        location.name = "forest".to_string();
        locations.push(location);
        the_greenwood.push(id);
    }

    for id in &the_greenwood {
        let exit_count = rng.gen_range(1, 4);
        let exits:Vec<&LocationId> = the_greenwood.choose_multiple(&mut rng, exit_count).collect();
        for exit in &exits {
            locations[**exit].exits.push(*id);
        }
        locations[*id].exits.extend(exits);
    }

    for village in &mut villages {
        let seed = *the_greenwood.choose(&mut rng).unwrap();
        locations[seed].name = "village".to_string();
        the_greenwood.remove_item(&seed);
        village.push(seed);
    }

    while locations.len() < location_count as usize {
        let village = villages.choose_mut(&mut rng).unwrap();
        let id = locations.len();

        let to_split_id = *village.choose(&mut rng).unwrap();
        let to_split_exits = (&locations[to_split_id]).exits.to_vec();
        
        let exits_a;
        let exits_b;

        if to_split_exits.len() == 1 {
            exits_a = to_split_exits.to_vec();
            exits_b = to_split_exits.to_vec();
        } else {
            let partition = to_split_exits.len() / 2;
            exits_a = to_split_exits[..partition].to_vec();
            exits_b = to_split_exits[partition..].to_vec();
        }

        let mut new_location = Location::new(id);
        for _ in 0..rng.gen_range(0, 15) {
            new_location.items.insert(item_id, Item {
                id: item_id,
                name: "carrot".to_string(),
                food_value: 10.0,
            });
            item_id += 1;
        }
        new_location.name = "village".to_string();
        for exit in &exits_a {
            locations[*exit].exits.push(to_split_id);
        }
        for exit in &exits_b {
            locations[*exit].exits.push(new_location.id);
        }
        new_location.exits = exits_b;

        let to_split = &mut locations[to_split_id];
        to_split.exits = exits_a;

        locations.push(new_location);
        village.push(id);
    }

    let mut agents = Vec::with_capacity(agent_count as usize);
    for id in 0..agent_count {
        let mut a = Agent::new(id as AgentId);
        let village = villages.choose_mut(&mut rng).unwrap();
        a.location = *village.choose(&mut rng).unwrap();
        agents.push(a);
    }


    (locations, agents)
}
