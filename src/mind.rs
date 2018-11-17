use std::collections::HashMap;
use rand::Rng;
use std::rc::Rc;
use rand::prelude::SliceRandom;
use super::conversation::simulate_conversation;

use super::{Health, Location, Event, ActionSource, Agent};

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

impl ActionSource  for Mind {
    fn actions(&self, agent: &Agent) -> Vec<(f64, fn(&mut Agent))> {
        let mut actions:Vec<(f64, fn(&mut Agent))> = Vec::new();
        if agent.health.awake {
            actions.push((1.0, |agent| {
                let mut rng = rand::thread_rng();
                let loc = agent.location.clone();
                let exits = loc.exits.borrow();
                let new_location = exits.choose_weighted(&mut rng, |loc| *agent.mind.opinions_on_places.get(loc.id).  unwrap_or(&0.0) + 10000.0).unwrap().clone();
                agent.events.push(Event { msg: format!("Moved from {} to {}", agent.location.id,
   new_location.id).to_string() });
                agent.location = new_location;
            }));
            actions.push((1.0, |agent| {
                let mut rng = rand::thread_rng();
                let mut people_i_care_about = Vec::new();
                  for a in agent.location.agents.borrow().iter() {
                      match a.try_borrow() {
                          Ok(aa) => {
                              if aa.health.alive &&
                                 aa.health.awake &&
                                 agent.mind.opinions_on_others.get(aa.id).unwrap_or(&0.0).abs() >
   0.4 {
                                  people_i_care_about.push(a.clone());
                              }
                          },
                          Err(_) => () // This is the current agent, fine.
                      }
                  }
                let who_should_i_talk_to = people_i_care_about.choose(&mut rng);
                  match who_should_i_talk_to {
                      Some(interlocular) => {
                          let conv = simulate_conversation(agent, &interlocular.borrow());
                          let mut interlocular = interlocular.borrow_mut();
                          agent.events.push(Event { msg: format!("Had {} with {} (initiated by me)", conv.to_string(), interlocular.name).to_string() });
                          interlocular.events.push(Event { msg: format!("Had {} with {} (initiated by them)", conv.to_string(), agent.name).to_string() });
                          agent.mind.cheer = (agent.mind.cheer + conv.tone * 0.01).max(-1.0).min(1.0);
                          interlocular.mind.cheer = (interlocular.mind.cheer + conv.tone * 0.01)
  .max(-1.0).min(1.0);
                      },
                      None => () // Nobody I care about around
                  };
            }));
        }
        actions
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

