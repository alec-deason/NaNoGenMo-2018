use std::borrow::Cow;
use std::io::Write;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use novel_gen::Agent;

type Nd = isize;
type Ed = (isize,isize,f64);
struct Edges(Vec<Ed>, HashMap<isize, String>);

pub fn render_to<W: Write>(agents: Vec<Rc<RefCell<Agent>>>, output: &mut W) {
    let mut edges = Vec::new();
    let mut labels = HashMap::with_capacity(agents.len());

    let mut weights = Vec::new();
    for a in &agents {
        for op in a.borrow().mind.opinions_on_others.iter() {
            weights.push((op.abs() * 1000.0) as u32);
        }
    }
    weights.sort();
    let thresh = weights[(weights.len() as f64 * 0.6) as usize] as f64 / 1000.0;

    for a in &agents {
        labels.insert(a.borrow().id as isize, a.borrow().name.clone());
        for (id, op) in a.borrow().mind.opinions_on_others.iter().enumerate() {
            if op.abs() > thresh {
                edges.push((a.borrow().id as isize, id as isize, *op));
            }
        }
    }
    let edges = Edges(edges, labels);
    dot::render(&edges, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", *n)).unwrap()
    }

    fn node_label(&'a self, n: &Nd) -> dot::LabelText<'a> {
        let labels = &self.1;
        dot::LabelText::label(labels[n].clone())
    }

    fn edge_color(&'a self, e: &Ed) -> Option<dot::LabelText<'a>> {
        if e.2 > 0.0 {
            Some(dot::LabelText::label("chartreuse"))
        } else {
            Some(dot::LabelText::label("crimson"))
        }
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges(ref v, _) = self;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s,t,_) in v {
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
