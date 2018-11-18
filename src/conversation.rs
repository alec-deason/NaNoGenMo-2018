use std::collections::HashMap;
use rand::prelude::SliceRandom;
use rand::prelude::IteratorRandom;

use super::{Agent};

pub struct Conversation {
    pub tone: f64,
    pub topics: Vec<Topic>,
}

impl Conversation {
      pub fn to_string(&self) -> String {
          let tone = if self.tone > 1.0 {
              "nice"
          } else if self.tone < -1.0 {
              "angry"
          } else { "neutral" };

          let mut topics = Vec::new();
          for topic in &self.topics {
              let topic_str = match topic {
                  Topic::Person {id} => "Some dude".to_string(), //w.agents[*id].borrow().name.clone(),
                  Topic::Relationship { id_a, id_b, tone } => {
                      "a relationship".to_string()
                  },
                  Topic::Thing { o_id, p_id } => "Some thing".to_string(), //w.items[*id].name.clone(),
                  Topic::Place { id } => "a place".to_string(),
                  Topic::Path { id_a, id_b } => "a path".to_string(),
                  Topic::SmallTalk => "nothing in particular".to_string(),
              };
              topics.push(topic_str);
          }
          let topics = topics.join(", ");

          format!("a {} conversation about {}", tone, topics).to_string()
      }
}

pub enum Topic {
    Person { id: usize },
    Relationship { id_a: usize, id_b: usize, tone: f64},
    Thing { o_id: usize, p_id: usize },
    Place { id: usize },
    Path { id_a: usize, id_b: usize },
    SmallTalk,
}

pub fn simulate_conversation(a: &Agent, b: &Agent) -> Conversation {
    let mut rng = rand::thread_rng();
    let a_b_tone = a.mind.opinions_on_others.get(b.id).unwrap_or(&0.0);
    let b_a_tone = b.mind.opinions_on_others.get(a.id).unwrap_or(&0.0);

    let topic = match ["people", "place", "path", "thing", "nothing"].choose(&mut rng).unwrap() {
        &"people" => who_can_we_talk_about(a, b),
        &"place" => where_can_we_talk_about(a, b),
        &"path" => what_path_can_we_talk_about(a, b),
        &"thing" => what_can_we_talk_about(a, b),
        _ => Topic::SmallTalk,
    };

    Conversation {
        tone: (a_b_tone + b_a_tone) / 2.0,
        topics: vec![topic],
    }
}

fn who_can_we_talk_about(a: &Agent, b: &Agent) -> Topic {
    let len = a.mind.opinions_on_others.len().max(b.mind.opinions_on_others.len()).max(a.id).max(b.id);
    let len = len + 1;
    let mut subject_weights = vec![0.0; len];
    for aa in &[a, b] {
        for (id, opinion) in aa.mind.opinions_on_others.iter().enumerate() {
            subject_weights[id] += opinion.abs();
        }
    }
    subject_weights[a.id] = 0.0;
    subject_weights[b.id] = 0.0;

    let total_weight:f64 = subject_weights.iter().sum();
    if total_weight > 0.0 {
        let mut rng = rand::thread_rng();
        let ids:Vec<usize> = (0..len).collect();
        Topic::Person { id: *ids.choose_weighted(&mut rng, |id| subject_weights[*id]).unwrap() }
    } else {
        Topic::SmallTalk
    }
}

fn what_can_we_talk_about(a: &Agent, b: &Agent) -> Topic {
    let mut rng = rand::thread_rng();
    let thing = a.mind.objects_seen.iter().chain(b.mind.objects_seen.iter()).choose(&mut rng);
    match thing {
        Some((o_id, p_id)) => Topic::Thing { o_id: *o_id, p_id: *p_id },
        None => Topic::SmallTalk,
    }
}

fn where_can_we_talk_about(a: &Agent, b: &Agent) -> Topic {
    let len = a.mind.opinions_on_places.len().max(b.mind.opinions_on_places.len()).max(a.id).max(b.id);
    let len = len + 1;
    let mut subject_weights = vec![0.0; len];
    for aa in &[a, b] {
        for (id, opinion) in aa.mind.opinions_on_places.iter().enumerate() {
            subject_weights[id] += opinion.abs();
        }
    }
    let total_weight:f64 = subject_weights.iter().sum();
    if total_weight > 0.0 {
        let mut rng = rand::thread_rng();
        let ids:Vec<usize> = (0..len).collect();
        Topic::Place { id: *ids.choose_weighted(&mut rng, |id| subject_weights[*id]).unwrap() }
    } else {
        Topic::SmallTalk
    }
}

fn what_path_can_we_talk_about(a: &Agent, b: &Agent) -> Topic {
    let mut subject_weights = HashMap::new();
    for aa in &[a, b] {
        for (id_a, connections) in aa.mind.location_edges.iter() {
            for id_b in connections {
                let opinion = aa.mind.opinions_on_places[*id_a].abs().max(aa.mind.opinions_on_places[*id_b].abs());
                subject_weights.insert((*id_a, *id_b), subject_weights.get(&(*id_a, *id_b)).unwrap_or(&0.0) + opinion);
            }
        }
    }
    let total_weight:f64 = subject_weights.values().sum();
    if total_weight > 0.0 {
        let mut rng = rand::thread_rng();
        let keys:Vec<&(usize, usize)> = subject_weights.keys().collect();
        match keys.choose_weighted(&mut rng, |k| subject_weights[k]) {
            Ok((id_a, id_b)) => Topic::Path { id_a: *id_a, id_b: *id_b },
            Err(_) => Topic::SmallTalk,
        }
    } else {
        Topic::SmallTalk
    }
}
