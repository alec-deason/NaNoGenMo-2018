use std::borrow::Cow;
use std::io::Write;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use novel_gen::{Location, World};

type Nd = isize;
type Ed = (isize,isize);
struct Edges(Vec<Ed>, HashMap<isize, f64>);

pub fn render_to<W: Write>(locations: &Vec<Rc<Location>>, world: &World, output: &mut W) {
    let mut edges = Vec::new();
    let mut labels = HashMap::with_capacity(locations.len());

    for loc in locations {
        let at = ((loc.agent_time.get() / world.agent_time) * 10000.0) as u64;
        labels.insert(loc.id as isize, at);
        for other in loc.exits.borrow().iter() {
            edges.push((loc.id as isize, other.id as isize));
        }
    }

    let mut real_labels = HashMap::with_capacity(labels.len());
    let max = *labels.values().max().unwrap() as f64;
    for (id, at) in &labels {
        let at = *at as f64 / max;
        real_labels.insert(*id, at);
    }
    let edges = Edges(edges, real_labels);
    dot::render(&edges, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", *n)).unwrap()
    }

    fn node_style(&'a self, _n: &Nd) -> dot::Style {
        dot::Style::Filled
    }

    fn node_color(&'a self, n: &Nd) -> Option<dot::LabelText<'a>> {
        let labels = &self.1;
        Some(dot::LabelText::label(format!("0.0 1.0 {}", labels[n]).to_string()))
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges(ref v, _) = self;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s,t) in v {
            nodes.push(s); nodes.push(t);
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a,Ed> {
        let &Edges(ref edges, _) = self;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Ed) -> Nd { e.0 }

    fn target(&self, e: &Ed) -> Nd { e.1 }
}
