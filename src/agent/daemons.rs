use std::cell::Cell;

use super::{Agent, Event, World};
use super::executive;
use super::events;

pub trait Daemon {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64>;
    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![]
    }
}

pub struct Wanderlust {
    last_wander: Cell<f64>,
}


impl Daemon for Wanderlust {
    fn step_simulation(&self, agent: &Agent, world: &World) -> Option<f64> {
        let min_wait = 5.0;
        let max_wait = 50.0;
        let wait = world.time - self.last_wander.get();

        if wait > min_wait {
            Some((wait / max_wait).min(1.0))
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        self.last_wander.set(world.time);
        vec![
            executive::wander(agent, world),
        ]
    }
}

pub struct HungerTracker;
impl Daemon for HungerTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.hunger += 0.1;

        if health.hunger > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::FindFood).or_insert(0.0);
            *goal += 0.5;
        }

        if health.hunger > 10.0 {
            health.pain += 0.1;
        }

        None
    }
}

pub struct SleepTracker;
impl Daemon for SleepTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.sleepiness += 0.1;

        if health.sleepiness > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::Rest).or_insert(0.0);
            *goal += 0.5;
        }

        if health.sleepiness > 10.0 {
            Some(10.0)
        } else {
            None
        }
    }

    fn events(&self, agent: &Agent, world: &World) -> Vec<Box<dyn Event>> {
        vec![
            Box::new(events::NapEvent { agent: agent.id })
        ]
    }
}

pub struct PoopTracker;
impl Daemon for PoopTracker {
    fn step_simulation(&self, agent: &Agent, _: &World) -> Option<f64> {
        let mut health = agent.health.borrow_mut();
        health.poop += 0.1;

        if health.poop > 1.0 {
            let mut mind = agent.mind.borrow_mut();
            let goal = mind.goals.entry(executive::Goal::Shit).or_insert(0.0);
            *goal += 0.5;
        }

        if health.poop > 2.0 {
            health.pain += 0.1;
        }

        if health.poop > 10.0 {
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
