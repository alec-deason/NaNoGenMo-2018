use rand::seq::SliceRandom;

use super::{Agent, World, Event};
use super::super::DummyEvent;
use super::events;
use super::daemons;

pub enum StrategyState {
    Complete { events: Vec<Box<dyn Event>> },
    Incomplete { events: Vec<Box<dyn Event>> },
}

pub trait Strategy {
    fn step_simulation(&mut self, agent: &Agent, world: &World) -> StrategyState;
}

pub struct FindSolitude {
    payload: Vec<Box<dyn Event>>,
}

impl Strategy for FindSolitude {
    fn step_simulation(&mut self, agent: &Agent, world: &World) -> StrategyState {
        let location = &world.locations[agent.location];
        if location.agents.len() > 1 {
            StrategyState::Incomplete { events: vec![
                Box::new(DummyEvent { agent: agent.id, message: "I'm not alone...".to_string() }),
                wander(agent, world),
            ]}
        } else {
            StrategyState::Complete { events: self.payload.drain(..).collect() }
        }
    }
}

pub fn wander(agent: &Agent, world: &World) -> Box<dyn Event> {
    let mut rng = rand::thread_rng();
    let new_loc = *world.locations[agent.location].exits.choose(&mut rng).unwrap_or(&agent.location);
    Box::new(events::MoveEvent { start: agent.location, end: new_loc, agent: agent.id })
}

pub struct FindFood { }

impl Strategy for FindFood {
    fn step_simulation(&mut self, agent: &Agent, world: &World) -> StrategyState {
        match agent.inventory.iter().find(|i| i.1.food_value > 0.0) {
            Some((id, _)) => {
                StrategyState::Complete { events: vec![
                    Box::new(events::EatEvent {
                        agent: agent.id,
                        item: *id,
                    }),
                ]}
            },
            None => {
                let location = &world.locations[agent.location];
                match location.items.iter().find(|i| i.1.food_value > 0.0) {
                    Some((id, _)) => {
                        StrategyState::Incomplete { events: vec![
                            Box::new(events::PickupEvent{
                                location: location.id,
                                agent: agent.id,
                                item: *id,
                            }),
                        ]}
                    },
                    None => {
                        StrategyState::Incomplete { events: vec![
                            Box::new(DummyEvent { agent: agent.id, message: "Nothing to eat here...".to_string() }),
                            wander(agent, world),
                        ]}
                    },
                }
            }
        }
    }
}

#[derive(Hash, PartialEq, Copy, Clone)]
pub enum Goal {
    FindFood,
    Rest,
    Shit,
}

impl Eq for Goal {}

pub struct Executive;
impl daemons::Daemon for Executive {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut rng = rand::thread_rng();

        let mut mind = agent.mind.borrow_mut();

        if mind.current_goal.is_none() {
            let goals: Vec<(&Goal, &f64)> = mind.goals.iter().collect();
            match goals.choose_weighted(&mut rng, |k| k.1) {
                Ok((k, _)) => {
                    match k {
                        Goal::FindFood => { mind.current_goal = Some((**k, Box::new(FindFood {}))); },
                        Goal::Shit => {
                            mind.current_goal = Some((**k, Box::new(FindSolitude { payload: vec![
                                Box::new(events::DefecateEvent { agent: agent.id }),
                            ]})));
                        },
                        Goal::Rest => {
                            mind.current_goal = Some((**k, Box::new(FindSolitude { payload: vec![
                                Box::new(events::NapEvent { agent: agent.id }),
                            ]})));
                        }
                    };
                    Some(1.0)
                },
                Err(_) => None,
            }
        } else {
            Some(1.0)
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        let mut mind = agent.mind.borrow_mut();
        match &mut mind.current_goal {
            Some((_, strategy)) => {
                match strategy.step_simulation(agent, world) {
                    StrategyState::Complete {  events } => {
                        mind.current_goal = None;
                        events
                    },
                    StrategyState::Incomplete { events } => events
                }
            },
            None => vec![]
        }
    }
}
