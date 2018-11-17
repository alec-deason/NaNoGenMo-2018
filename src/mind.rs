use std::collections::HashMap;
use rand::Rng;
use std::boxed::FnBox;
use std::rc::Rc;

use super::{Health, Location, Event, ActionSource, ActionView, PreActionView};

pub struct Mind {
    pub cheer: f64,
    pub disposition: f64,
    pub opinions_on_others: Vec<f64>,
    pub preconceptions: Vec<f64>,
    pub opinions_on_places: Vec<f64>,
    pub objects_seen: HashMap<usize, usize>,
}

pub struct MindView<'a> {
    pub health: &'a Health,
    pub location: Rc<Location>,
    pub events: &'a mut Vec<Event>,
}

impl<'a> ActionSource<'a>  for Mind {
    fn actions(&'a self, view: &PreActionView) -> Vec<(f64, Box<FnBox(&mut ActionView) + 'a>)> {
        Vec::new()
  }
}
impl Mind {
    pub fn new() -> Mind {
        let mut rng = rand::thread_rng();
        let disposition = rng.gen_range(-1.0, 1.0);
        Mind {
            cheer: disposition,
            disposition: disposition,
            opinions_on_others: Vec::with_capacity(1000),
            preconceptions: Vec::with_capacity(1000),
            opinions_on_places: Vec::with_capacity(1000),
            objects_seen: HashMap::with_capacity(1000),
        }
    }

    pub fn step_simulation(&mut self, view: MindView) {
        let mut rng = rand::thread_rng();
        // Current cheer level tends to drift back towards overall disposition
        self.cheer += -0.001*(self.cheer-self.disposition);
        self.cheer = (self.cheer - view.health.pain * 0.2).min(1.0).max(-1.0);

        if view.health.awake {
            if view.health.sleepiness > 16.0 {
                self.cheer -= 0.1;
            }

            let d_opinion = self.cheer*0.1;
            for a in view.location.agents.borrow().iter() {
                match a.try_borrow() {
                    Ok(a) => {
                        if a.health.alive {
                            while self.opinions_on_others.len() <= a.id {
                                self.preconceptions.push(rng.gen_range(-0.1, 0.1));
                                self.opinions_on_others.push(0.0)
                            }
                            if self.opinions_on_others[a.id] == 0.0 {
                                view.events.push(Event { msg: format!("Met {}", a.name).to_string() });
                            }
                            self.opinions_on_others[a.id] += d_opinion + self.preconceptions[a.id];
                        }
                    },
                    Err(_) => () // This is the current agent, fine.
                }
            }

            for o in view.location.items.borrow().iter() {
                self.objects_seen.insert(o.id, view.location.id);
            }

            
            while self.opinions_on_places.len() <= view.location.id {
                self.opinions_on_places.push(rng.gen_range(-0.001, 0.001));
            }
            self.opinions_on_places[view.location.id] += d_opinion;
        }
    }
}

