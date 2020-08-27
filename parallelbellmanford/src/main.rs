use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cmp::{max,min};


// Graph struct inspired by Niko Matsakis's post on Baby Steps
pub struct Graph {
    nodes : Vec<NodeData>,
    edges : Vec<EdgeData>, //ingoing edges
}

pub type NodeIndex = usize;
pub type EdgeIndex = usize;

pub struct NodeData {
    first_ingoing_edge : Option<EdgeIndex>,
}

pub struct EdgeData {
    source : NodeIndex,
    next_ingoing_edge : Option<EdgeIndex>,
    poids : i8,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes : Vec::new(),
            edges : Vec::new(),
        }
    }

    pub fn add_node(&mut self) -> NodeIndex {
        let new: NodeIndex = self.nodes.len();
        self.nodes.push(NodeData{first_ingoing_edge: None});
        new
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, poids: i8) {
        let new : EdgeIndex = self.edges.len();
        self.edges.push(EdgeData{
            source: source,
            next_ingoing_edge: self.nodes[target].first_ingoing_edge,
            poids: poids,
        });
        self.nodes[target].first_ingoing_edge = Some(new)
    }

    pub fn predecessors(&self, target: NodeIndex) -> Predecessors {
        let first_ingoing_edge = self.nodes[target].first_ingoing_edge;
        Predecessors{
            graph : &self,
            current_edge_index: first_ingoing_edge,
        }
    }
}

pub struct Predecessors<'graph> {
    graph: &'graph Graph,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph> Iterator for Predecessors<'graph> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<EdgeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge_data = &self.graph.edges[edge_num];
                self.current_edge_index = edge_data.next_ingoing_edge;
                Some(edge_num)
            }
        }
    }
}

impl Graph {
    // return true if a negative cycle is detected.
    pub fn parallel_bellman_ford(&self) -> (Vec<i8>, bool) {
        let mut best_distance : Vec<i8> = (0..self.nodes.len())
                                        .map(|i| if i!=0 {std::i8::MAX} else {0})
                                        .collect();
        for _ in 0..self.nodes.len() {
            let new_best_distance : Vec<(usize,i8)> = (0..self.nodes.len())
                .into_par_iter()
                .map(|i| {
                    let mut new_at_index_i = best_distance[i];
                    for predecessor_edge in self.predecessors(i) {
                        let edge = &self.edges[predecessor_edge];
                        let proposition = avoid_overflow(best_distance[edge.source], edge.poids);
                        new_at_index_i = min(new_at_index_i, proposition);
                    }
                    (i, new_at_index_i)
                    //pas sur qu'il y ait vraiment besoin de spécifier l'indice
                }).collect();
            for (index, value_at_index) in new_best_distance {
                best_distance[index] = value_at_index;
            }
        }
        let negative_cycle = AtomicBool::new(false);
        (0..self.nodes.len())
        .into_par_iter()
        .for_each(|i| {
            let mut new_at_index_i = best_distance[i];
            for predecessor_edge in self.predecessors(i) {
                let edge = &self.edges[predecessor_edge];
                let proposition = avoid_overflow(best_distance[edge.source], edge.poids);
                new_at_index_i = min(new_at_index_i, proposition);
            }
            if new_at_index_i<best_distance[i] {negative_cycle.store(true, Ordering::SeqCst)}
        });
        (best_distance, negative_cycle.into_inner())
    }
}

fn avoid_overflow(a: i8, b: i8) -> i8 {
    let (small, big) = (min(a,b), max(a,b));
    if (small == std::i8::MIN) & (big<0) {
        std::i8::MIN
    } else if (small>0) & (big==std::i8::MAX) {
        std::i8::MAX
    } else {
        a+b
    }
} 

fn main() {
    let mut graph = Graph::new();
    for _ in 0..4 {
        graph.add_node();
    }
    graph.add_edge(0, 1, 1);
    graph.add_edge(0, 2, 2);
    graph.add_edge(1, 2, 0);
    graph.add_edge(1, 3, 7);
    graph.add_edge(2, 3, 3);
    graph.add_edge(3, 0, 4);
    println!("{:?}", graph.parallel_bellman_ford());

    let mut graph2 = Graph::new();
    for _ in 0..10000 {
        graph2.add_node();
    }
    for i in 0..10000 {
        graph2.add_edge(i, (i+1)%1000, 1);
    }
    let (_,negative_cycle) = graph2.parallel_bellman_ford();
    println!("{:?}", negative_cycle);
}
