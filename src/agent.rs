use rand::seq::SliceRandom;
use rand::seq::IteratorRandom;

use super::{Event, DummyEvent, Item, World, LocationId};

pub type AgentId = usize;
pub struct Agent {
    id: AgentId,
    cheer: f64,
    pub location: usize,
    pub events: Vec<Box<dyn Event>>,
    inventory: Vec<Item>,

    wanderlust: Wanderlust,
    nose_picker: NosePicker,
}

impl Agent {
    pub fn new(id: AgentId) -> Agent {
        Agent {
            id: id,
            cheer: 0.0,
            location: 0,
            events: Vec::new(),
            inventory: Vec::new(),

            wanderlust: Wanderlust { last_wander: 0.0 },
            nose_picker: NosePicker {},
        }
    }

    pub fn step_simulation(&self, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        let daemons:Vec<&dyn Daemon> = vec![&self.wanderlust, &self.nose_picker];
        let mut daemon_urgency: Vec<f64> = Vec::with_capacity(daemons.len());
        let mut potential_daemons = Vec::with_capacity(daemons.len());

        for daemon in &daemons {
            match daemon.urgency(self, world) {
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
    fn urgency(&self, agent: &Agent, world: &World) -> Option<f64>;
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>>;
}

struct Wanderlust {
    last_wander: f64,
}

struct UpdateWanderlustEvent {
    agent: AgentId,
}

impl Event for UpdateWanderlustEvent {
    fn apply(&self, world: &mut World) {
        world.agents[self.agent].wanderlust.last_wander = world.time;
    }

    fn to_string(&self, world: &World) -> String {
        "".to_string()
    }
}


impl Daemon for Wanderlust {
    fn urgency(&self, agent: &Agent, world: &World) -> Option<f64> {
        let min_wait = 5.0;
        let max_wait = 50.0;
        let wait = world.time - self.last_wander;
        
        if wait > min_wait {
            Some((wait / max_wait).min(1.0))
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        let mut rng = rand::thread_rng();
        let new_loc = *world.locations[agent.location].exits.choose(&mut rng).unwrap_or(&agent.location);
        if new_loc != agent.location {
            vec![
                Box::new(MoveEvent { start: agent.location, end: new_loc, agent: agent.id }),
                Box::new(UpdateWanderlustEvent { agent: agent.id }),
            ]
        } else {
            vec![]
        }
    }
}

struct NosePicker;
impl Daemon for NosePicker {
    fn urgency(&self, _: &Agent, _: &World) -> Option<f64> {
        Some(0.001)
    }

    fn events(&self, agent: &Agent, _: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(DummyEvent {
                agent: agent.id,
                message: "Picks nose.".to_string()
            }),
        ]
    }
}
