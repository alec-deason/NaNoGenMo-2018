use std::cell::Cell;
use std::collections::hash_map::Entry;

use super::{Agent, AgentId, Event, World};
use super::executive;
use super::events;

pub trait Daemon {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64>;
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![]
    }
}

pub struct Wanderlust {
    pub last_wander: Cell<f64>,
}


impl Daemon for Wanderlust {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64> {
        let min_wait = 5.0;
        let max_wait = 50.0;
        let wait = world.time - self.last_wander.get();

        if wait > min_wait {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::Explore).or_insert(0.0);
            *goal = (wait / max_wait).min(0.5);
        }
        None
    }
}

pub struct HungerTracker;
impl Daemon for HungerTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        if health.awake {
            health.hunger += 1.0;
        } else {
            health.hunger += 0.25;
        }

        if health.hunger > 5.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::FindFood).or_insert(0.0);
            *goal += 0.5;
        }

        if health.hunger > 10.0 {
            health.pain += 0.1;
        }


        if health.hunger > 24.0 * 30.0 {
            Some(10.0)
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(events::DieEvent { agent: agent.id })
        ]
    }
}

pub struct SleepTracker;
impl Daemon for SleepTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        if health.awake {
            health.sleepiness += 1.0/16.0;
        } else {
            health.sleepiness -= 1.0/8.0;
            if health.sleepiness <= 0.0 {
                return Some(1.0);
            }
        }

        let mut mind = agent.mind.borrow_mut();
        if health.sleepiness > 1.0 {
            let goal = mind.goals.entry(executive::Goal::Rest).or_insert(0.0);
            *goal += 0.5;
        } else {
            let goal = mind.goals.remove(&executive::Goal::Rest);
        }

        if health.sleepiness > 24.0 {
            Some(10.0)
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        if agent.health.borrow().sleepiness > 1.0 {
            vec![
                Box::new(events::NapEvent { agent: agent.id })
            ]
        } else {
            vec![
                Box::new(events::WakeEvent { agent: agent.id })
            ]
        }
    }
}

pub struct PoopTracker;
impl Daemon for PoopTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        if health.hunger < 12.0 {
            if health.awake {
                health.poop += 1.0;
            } else {
                health.poop += 0.25;
            }
        }

        if health.poop > 4.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::Shit).or_insert(0.0);
            *goal += 0.5;
        }

        if health.poop > 6.0 {
            health.pain += 0.1;
        }

        if health.poop > 12.0 {
            Some(10.0)
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(events::DefecateEvent { agent: agent.id })
        ]
    }
}

pub struct PainTracker;
impl Daemon for PainTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let health = agent.health.borrow_mut();

        if health.pain > 0.0 {
            let mut mind = agent.mind.borrow_mut();
            mind.agitation += 0.1;
            mind.cheer -= 0.1;
        }
        None
    }
}

pub struct EncounterTracker {
    encounter: Cell<Option<AgentId>>,
}
impl EncounterTracker {
    pub fn new() -> EncounterTracker {
        EncounterTracker {
            encounter: Cell::new(None),
        }
    }
}
impl Daemon for EncounterTracker {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64> {
        let mut mind = agent.mind.borrow_mut();
        let cheer = mind.cheer;
        for a in &world.locations[agent.location].agents {
            if (*a != agent.id) & !mind.opinions_on_others.contains_key(a) {
                self.encounter.set(Some(*a));
                return Some(0.25);
            }
        }
        None
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(events::MeetEvent { agent: agent.id, other: self.encounter.get().unwrap() })
        ]
    }
}
