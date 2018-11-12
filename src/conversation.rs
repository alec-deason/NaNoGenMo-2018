use std::fmt;
use rand::prelude::SliceRandom;

use super::Agent;

pub struct Conversation {
    pub tone: f64,
    pub topics: Vec<Topic>,
}

impl fmt::Display for Conversation {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
          let tone = if self.tone > 1.0 {
              "nice"
          } else if self.tone < -1.0 {
              "angry"
          } else { "neutral" };

          let mut topics = Vec::new();
          for topic in &self.topics {
              let topic_str = match topic {
                  Topic::Person {id} => "a person",
                  Topic::Relationship { id_a, id_b, tone } => "a relationship",
                  Topic::SmallTalk => "nothing in particular",
              };
              topics.push(topic_str);
          }
          let topics = topics.join(", ");

          write!(f, "a {} conversation about {}", tone, topics)
      }
}

pub enum Topic {
    Person { id: usize },
    Relationship { id_a: usize, id_b: usize, tone: f64},
    SmallTalk,
}

pub fn simulate_conversation(a: &Agent, b: &Agent) -> Conversation {
    let a_b_tone = a.mind.opinions_on_others.get(b.id).unwrap_or(&0.0);
    let b_a_tone = b.mind.opinions_on_others.get(a.id).unwrap_or(&0.0);

    let topic = match who_can_we_talk_about(a, b) {
        Some(id) => Topic::Person {id},
        None => Topic::SmallTalk,
    };

    Conversation {
        tone: (a_b_tone + b_a_tone) / 2.0,
        topics: vec![topic],
    }
}

fn who_can_we_talk_about(a: &Agent, b: &Agent) -> Option<usize> {
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
        Some(*ids.choose_weighted(&mut rng, |id| subject_weights[*id]).unwrap())
    } else {
        None
    }
}
