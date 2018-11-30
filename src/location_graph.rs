use std::borrow::Cow;
use std::io::Write;
use std::collections::HashMap;

use novel_gen::{Location, World};

type Nd = isize;
type Ed = (isize,isize);
struct Edges(Vec<Ed>, HashMap<isize, bool>);

pub fn render_to<W: Write>(locations: &Vec<Location>, world: &World, output: &mut W) {
    let mut edges = Vec::new();
    let mut labels = HashMap::with_capacity(locations.len());

    for loc in locations {
        let at = loc.name.contains("village");
        labels.insert(loc.id as isize, at);
        for other in loc.exits.iter() {
            edges.push((loc.id as isize, *other as isize));
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

    fn node_style(&'a self, _n: &Nd) -> dot::Style {
        dot::Style::Filled
    }

    fn node_color(&'a self, n: &Nd) -> Option<dot::LabelText<'a>> {
        let labels = &self.1;
        let red = if labels[n] { "ff" } else { "00" };
        let green = if labels[n] { "00" } else { "ff" };
        let l = format!("#{}{}00", red, green).to_string();
        Some(dot::LabelText::label(l))
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
