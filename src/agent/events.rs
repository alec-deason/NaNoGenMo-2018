use super::{Event, World, AgentId, LocationId};
use super::super::DummyEvent;
use super::executive;

#[derive(Copy, Clone)]
pub struct MoveEvent {
    pub start: LocationId,
    pub end: LocationId,
    pub agent: AgentId,
}

impl Event for MoveEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];

        world.locations[self.start].agents.remove_item(&self.agent);
        world.locations[self.end].agents.push(self.agent);
        agent.location = self.end;
        agent.events.push(Box::new(*self));

        let mut mind = agent.mind.borrow_mut();
        let cheer = mind.cheer;
        let opinion = mind.opinions_on_places.entry(self.start).or_insert(0.0);
        *opinion += cheer / 10.0
    }

    fn to_string(&self, world: &World) -> String {
        let start = &world.locations[self.start].name;
        let end = &world.locations[self.end].name;

        format!("Moved from {} to {}.", start, end).to_string()
    }
}

pub struct PickupEvent {
    pub location: LocationId,
    pub item: LocationId,
    pub agent: AgentId,
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
                agent.inventory.insert(item.id, item);
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

pub struct EatEvent {
    pub item: LocationId,
    pub agent: AgentId,
}

impl Event for EatEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];

        match agent.inventory.remove_entry(&self.item) {
            Some((_, item)) => {
                agent.events.push(Box::new(DummyEvent {
                    agent: agent.id,
                    message: format!("Ate {}.", item.name).to_string(),
                }));
                let mut health = agent.health.borrow_mut();
                health.hunger = (health.hunger - item.food_value).max(0.0);

                if health.hunger < 5.0 {
                    let mut mind = agent.mind.borrow_mut();
                    mind.goals.remove(&executive::Goal::FindFood);
                }
            },
            None => {
                agent.events.push(Box::new(DummyEvent {
                    agent: agent.id,
                    message: "Tried to eat something up but it wasn't there".to_string(),
                }));
            }
        }
    }

    fn to_string(&self, world: &World) -> String {
        "Trying to eat something.".to_string()
    }
}

#[derive(Copy, Clone)]
pub struct NapEvent {
    pub agent: AgentId,
}
impl Event for NapEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));
        let mut health = agent.health.borrow_mut();
        health.awake = false;
    }
    fn to_string(&self, world: &World) -> String {
        format!("Went to sleep.").to_string()
    }
}

#[derive(Copy, Clone)]
pub struct WakeEvent {
    pub agent: AgentId,
}
impl Event for WakeEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));
        let mut health = agent.health.borrow_mut();
        health.awake = true;
    }
    fn to_string(&self, world: &World) -> String {
        format!("Woke up.").to_string()
    }
}

#[derive(Copy, Clone)]
pub struct MeetEvent {
    pub agent: AgentId,
    pub other: AgentId,
}
impl Event for MeetEvent {
    fn apply(&self, world: &mut World) {
        {
            let agent = &mut world.agents[self.agent];
            agent.events.push(Box::new(self.clone()));
            let mut mind = agent.mind.borrow_mut();
            let cheer = mind.cheer;
            let o = mind.opinions_on_others.entry(self.other).or_insert(0.0);
            *o += cheer;
        }

        {
            let agent = &mut world.agents[self.other];
            let mut mind = agent.mind.borrow_mut();
            let cheer = mind.cheer;
            let o = mind.opinions_on_others.entry(self.agent).or_insert(0.0);
            *o += cheer;
        }
    }
    fn to_string(&self, world: &World) -> String {
        let other = &world.agents[self.other];
        format!("Met {}", other.name).to_string()
    }
}

#[derive(Copy, Clone)]
pub struct DieEvent {
    pub agent: AgentId,
}
impl Event for DieEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));
        let mut health = agent.health.borrow_mut();
        health.alive = false
    }
    fn to_string(&self, world: &World) -> String {
        format!("Died.").to_string()
    }
}

#[derive(Copy, Clone)]
pub struct DefecateEvent {
    pub agent: AgentId,
}
impl Event for DefecateEvent {
    fn apply(&self, world: &mut World) {
        let agent = &mut world.agents[self.agent];
        agent.events.push(Box::new(self.clone()));

        let mut health = agent.health.borrow_mut();
        health.poop = 0.0;

        let mut mind = agent.mind.borrow_mut();
        mind.goals.remove(&executive::Goal::Shit);

        *world.metrics.entry("shit").or_insert(0) += 1;
    }
    fn to_string(&self, _: &World) -> String {
        format!("Took a shit.").to_string()
    }
}
