use rand::seq::SliceRandom;

use super::{Agent, Mind, ItemId, World, Event};
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
    payload: fn(agent: &Agent, world: &World) -> Vec<Box<dyn Event>>,
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
            StrategyState::Complete { events: (self.payload)(agent, world) }
        }
    }
}

pub fn wander(agent: &Agent, world: &World) -> Box<dyn Event> {
    let mut rng = rand::thread_rng();
    let new_loc = *world.locations[agent.location].exits.choose(&mut rng).unwrap_or(&agent.location);
    Box::new(events::MoveEvent { start: agent.location, end: new_loc, agent: agent.id })
}

pub struct FindFood {
    payload: fn(item: &ItemId, agent: &Agent, world: &World) -> Vec<Box<dyn Event>>,
}

impl Strategy for FindFood {
    fn step_simulation(&mut self, agent: &Agent, world: &World) -> StrategyState {
        match agent.inventory.iter().find(|i| i.1.food_value > 0.0) {
            Some((id, _)) => {
                StrategyState::Complete { events: (self.payload)(id, agent, world) }
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

pub struct Explore {
    iterations: u32,
    payload: fn(agent: &Agent, world: &World) -> Vec<Box<dyn Event>>,
}

impl Strategy for Explore {
    fn step_simulation(&mut self, agent: &Agent, world: &World) -> StrategyState {
        self.iterations -= 1;
        
        if self.iterations > 0 {
            StrategyState::Incomplete { events: vec![wander(agent, world)] }
        } else {
            StrategyState::Complete { events: (self.payload)(agent, world) }
        }
    }
}

#[derive(Hash, PartialEq, Copy, Clone)]
pub enum Goal {
    FindFood,
    Rest,
    Shit,
    Explore,
}

impl Eq for Goal {}

fn choose_goal(mind: &mut Mind) -> bool {
    let mut rng = rand::thread_rng();
    let goals: Vec<(&Goal, &f64)> = mind.goals.iter().collect();
    match goals.choose_weighted(&mut rng, |k| k.1) {
        Ok((k, _)) => {
            match mind.paused_goals.iter().position(|i| i.0 == **k) {
                Some(i) => {
                    mind.current_goal = Some(mind.paused_goals.remove(i));
                    true
                },
                None => {
                    match k {
                        Goal::FindFood => { mind.current_goal = Some((**k, Box::new(FindFood {
                            payload: |item, agent, _| {
                                vec![
                                    Box::new(events::EatEvent {
                                        agent: agent.id,
                                        item: *item,
                                    }),
                                ]
                            }
                        }))); },
                        Goal::Shit => {
                            mind.current_goal = Some((**k, Box::new(FindSolitude { payload: 
                                |agent, _| {
                                    vec![
                                        Box::new(events::DefecateEvent { agent: agent.id }),
                                    ]}
                                })));
                        },
                        Goal::Rest => {
                            mind.current_goal = Some((**k, Box::new(FindSolitude { payload:
                                |agent, _| {
                                    vec![
                                        Box::new(events::NapEvent { agent: agent.id }),
                                    ]}
                                })));
                        }
                        Goal::Explore => {
                            mind.current_goal = Some((**k, Box::new(Explore { 
                                iterations: 5,
                                payload: |agent, _| {
                                    vec![
                                    ]}
                                })));
                        }
                    };
                    true
                },
            }
        },
        Err(_) => false,
    }
}
pub struct Executive;
impl daemons::Daemon for Executive {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        if !agent.health.borrow().awake {
            return None;
        }

        let mut rng = rand::thread_rng();

        let mut mind = agent.mind.borrow_mut();

        if mind.current_goal.is_none() {
            if mind.paused_goals.len() > 0 {
                let idxs:Vec<usize> = (0..mind.paused_goals.len()).into_iter().collect();
                let restart_goal = idxs.choose_weighted(&mut rng, |k| mind.goals[&mind.paused_goals[*k].0]).unwrap();
                mind.current_goal = Some(mind.paused_goals.remove(*restart_goal));
                Some(1.0)
            } else {
                if choose_goal(&mut mind) {
                    Some(1.0)
                } else {
                    None
                }
            }
        } else {
            let max_goal = mind.goals.values().cloned().fold(-1./0. , f64::max);
            if max_goal > mind.goals.get(&mind.current_goal.as_ref().unwrap().0).unwrap_or(&0.0) * 1.20 {
                choose_goal(&mut mind);
            }
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
