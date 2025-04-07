use clap::Parser;
use csv::ReaderBuilder;
use serde::{Deserialize};
use std::collections::{HashMap};
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::{Rc};
use std::option::Option;

#[derive(Debug, Deserialize)]
struct EdgeTable {
    A: u32,
    B: u32,
    id: u32,
}

#[derive(Clone)] // Derive Clone for Node struct
struct Edge {
    id: u32,
    start: Rc<RefCell<Node>>, // Reference to Node using Rc and RefCell
    end: Rc<RefCell<Node>>,   // Reference to Node using Rc and RefCell
}

#[derive(Clone)] // Derive Clone for Node struct
struct Node {
    id: u32,
    starts: Vec<Rc<RefCell<Edge>>>, // List of Edges starting from this Node
    ends: Vec<Rc<RefCell<Edge>>>,   // List of Edges ending at this Node
}

impl Node {
    fn new(id: u32) -> Self {
        Node {
            id,
            starts: Vec::new(),
            ends: Vec::new(),
        }
    }

    fn add_edge(&mut self, edge: Rc<RefCell<Edge>>, is_start: bool) {
        if is_start {
            self.starts.push(edge);
        } else {
            self.ends.push(edge);
        }
    }
}

impl Edge {
    fn new(id: u32, start: Rc<RefCell<Node>>, end: Rc<RefCell<Node>>) -> Self {
        Edge { id, start, end }
    }
}


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    start: u32,

    #[arg(long, default_value = "1")]
    min_nodes: u16,

    #[arg(long, default_value = "0")]
    max_nodes: u16,

    #[arg(long, default_value = "0")]
    max_paths: u32
}

fn create_graph(base_point: &Rc<RefCell<Node>>, map_nodes: &mut HashMap<u32, RefCell<Rc<RefCell<Node>>>>, mut map_edges: HashMap<u32, RefCell<Rc<RefCell<Edge>>>>, mut raw_edges: Vec<(u32, u32, u32)>) {
    // println!("\n=================\nCreating graph");
    let mut edges_added: u32 = 0;
    let mut raw_edges_new: Vec<(u32, u32, u32)> = Vec::new();
    loop {
        raw_edges_new = Vec::new();
        let mut added_something = false;
        for (a, b, eid) in &raw_edges {
            let mut insert_nnode_a = false;
            let mut insert_nnode_b = false;
            let mut nnode_a: Rc<RefCell<Node>> = Rc::clone(&base_point);
            let mut nnode_b: Rc<RefCell<Node>> = Rc::clone(&base_point);

            let mut node_a_opt: Option<&mut RefCell<Rc<RefCell<Node>>>>;
            let mut node_b_opt: Option<&mut RefCell<Rc<RefCell<Node>>>>;
            unsafe {
                [node_a_opt, node_b_opt] = map_nodes.get_disjoint_unchecked_mut([a, b]);
            }

            // Proceed only if at least one of the nodes exists
            if node_a_opt.is_none() || node_b_opt.is_some() {
                // Handling node_a
                let mut node_a = if let Some(node_a_rc) = node_a_opt {
                    // If node_a exists, check if the edge already exists
                    if map_edges.get(eid).is_some() {
                        continue; // Skip if edge already exists
                    }
                    node_a_rc // Otherwise, use the existing node_a
                } else {
                    // If node_a doesn't exist, create a new one
                    nnode_a = Rc::new(RefCell::new(Node::new(*a)));
                    insert_nnode_a = true;
                    &RefCell::from(Rc::clone(&nnode_a))
                };

                // Handling node_b
                let mut node_b = if let Some(node_b_rc) = node_b_opt {
                    // If node_b exists, check if the edge already exists
                    if map_edges.get_mut(eid).is_some() {
                        continue; // Skip if edge already exists
                    }
                    node_b_rc // Otherwise, use the existing node_b
                } else {
                    // If node_b doesn't exist, create a new one
                    nnode_b = Rc::new(RefCell::new(Node::new(*b)));
                    insert_nnode_b = true;
                    &RefCell::from(Rc::clone(&nnode_b))
                };

                // Create the edge
                let new_edge = Rc::new(RefCell::new(Edge::new(*eid, Rc::clone(&node_a.borrow()), Rc::clone(&node_b.borrow()))));
                map_edges.insert(*eid, RefCell::from(Rc::clone(&new_edge)));

                // Add the edge to both nodes
                let is_start_a = Rc::ptr_eq(&node_a.borrow(), &new_edge.borrow().start);
                node_a.borrow().borrow_mut().add_edge(Rc::clone(&new_edge), is_start_a);
                let is_start_b = Rc::ptr_eq(&node_b.borrow(), &new_edge.borrow().start);
                node_b.borrow().borrow_mut().add_edge(Rc::clone(&new_edge), is_start_b);

                edges_added += 1;
                added_something = true;
                if edges_added % 10000 == 0 {
                    // println!("edges added: {}", edges_added);
                }
            } else {
                raw_edges_new.push((*eid, *eid, *eid));
            }

            if insert_nnode_a {
                if !map_nodes.contains_key(a) {
                    map_nodes.insert(*a, RefCell::from(Rc::clone(&nnode_a)));
                }
            }
            if insert_nnode_b {
                if !map_nodes.contains_key(b) {
                    map_nodes.insert(*b, RefCell::from(Rc::clone(&nnode_b)));
                }
            }
        }
        if !added_something {
            break;
        } else {
            // println!("Walkthrough X finished. Edges now: {}", edges_added);
        }
        raw_edges = raw_edges_new;
    }

    // println!("Finished, edges added: {}", edges_added);
    // println!("Graph created\n=================\n");
}

fn dfs1(current_node: Rc<RefCell<Node>>, visited: Vec<u32>, edges_used: Vec<u32>, target_id: u32, min_nodes: u16, max_nodes: u16, max_paths: u32, paths_found: &mut u32) {
    if edges_used.len() >= max_nodes as usize {
        return;
    }

    if max_paths != 0 && *paths_found >= max_paths {
        return;
    }

    let current_id = current_node.borrow().id;
    if current_id == target_id && visited.len() > 1 && visited.len() >= (min_nodes - 1) as usize {
        // we got this, this is the end...
        *paths_found += 1;
        for edge in &edges_used {
            print!("{} ", edge);
        }
        print!("\n");
    }

    if visited.contains(&current_id) && current_id != target_id {
        return
    }

    let mut edges_from_there = current_node.borrow().starts.clone();
    edges_from_there.retain(|edge| !edges_used.contains(&edge.borrow().id));

    for edge in edges_from_there {
        let edge_id = edge.borrow().id;
        let next_node = edge.borrow().end.clone();
        dfs1(next_node,
             visited.iter().copied().chain(std::iter::once(current_id)).collect(),
             edges_used.iter().copied().chain(std::iter::once(edge_id)).collect(),
             target_id,
             min_nodes,
             max_nodes,
             max_paths,
             paths_found);
    }
}

fn find_elementary_cycles(the_vertex: Rc<RefCell<Node>>, min_nodes: u16, max_nodes: u16, max_paths: u32) {
    let mut paths_found: u32 = 0;
    let max_nodes_r = if max_nodes == 0 {u16::MAX} else {max_nodes};
    let target_id = the_vertex.borrow().id;
    dfs1(the_vertex, Vec::new(), Vec::new(), target_id, min_nodes, max_nodes_r, max_paths, &mut paths_found);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(&args.input)?;

    let mut raw_edges: Vec<(u32, u32, u32)> = Vec::new();  // Ukládáme hrany jako (u32, u32, u32)

    for result in rdr.deserialize() {
        let record: EdgeTable = result?;

        raw_edges.push((record.A, record.B, record.id));
    }

    let num_nodes = raw_edges.iter().flat_map(|(a, b, _)| vec![a, b]).max().unwrap_or(&0) + 1;
    let num_edges = raw_edges.len() as u32;

    let min_nodes = args.min_nodes;
    let max_nodes = args.max_nodes;
    let max_paths = args.max_paths;
    let mut base_point = Rc::new(RefCell::new(Node::new(args.start)));
    let mut map_nodes: HashMap<u32, RefCell<Rc<RefCell<Node>>>> = HashMap::new();
    let mut map_edges: HashMap<u32, RefCell<Rc<RefCell<Edge>>>> = HashMap::new();

    // println!("num_edges: {}", num_edges);
    // println!("num_nodes: {}", num_nodes);

    create_graph(&base_point, &mut map_nodes, map_edges, raw_edges);

    let new_base_point = map_nodes.get(&base_point.borrow().id).unwrap().borrow().clone();

    find_elementary_cycles(new_base_point, min_nodes, max_nodes, max_paths);

    Ok(())
}