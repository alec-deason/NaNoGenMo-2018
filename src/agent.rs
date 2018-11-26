use rand::seq::SliceRandom;

use super::{Event, DummyEvent, Item, World, LocationId};

pub type AgentId = usize;
pub struct Agent {
    id: AgentId,
    cheer: f64,
    pub location: usize,
    pub events: Vec<Box<dyn Event>>,
    inventory: Vec<Item>,
}

impl Agent {
    pub fn new(id: AgentId) -> Agent {
        Agent {
            id: id,
            cheer: 0.0,
            location: 0,
            events: Vec::new(),
            inventory: Vec::new(),
        }
    }

    pub fn step_simulation(&self, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        let new_loc = *world.locations[self.location].exits.choose(&mut rng).unwrap_or(&self.location);
        if new_loc != self.location {
            vec![Box::new(MoveEvent { start: self.location, end: new_loc, agent: self.id })]
        } else {
            vec![]
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
                    message: format!("Picked up {}.", item.name).to_string(),
                }));
                agent.inventory.push(item);
            },
            None => {
                agent.events.push(Box::new(DummyEvent {
                    message: "Tried to pick something up but it wasn't there".to_string(),
                }));
            }
        }
    }

    fn to_string(&self, world: &World) -> String {
        "Trying to pick something up.".to_string()
    }
}
