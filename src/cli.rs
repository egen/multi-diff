use anyhow::*;
use clap::{clap_app, crate_version, App};
use serde_yaml::*;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::Path,
    rc::Rc,
};

fn app() -> App<'static, 'static> {
    clap_app!(cepler =>
        (version: crate_version!())
        (@setting VersionlessSubcommands)
        (@arg FILES: ... * "Input files")
    )
}

fn load_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Value> {
    let file = File::open(path).context("Couldn't open file")?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader)?)
}

pub fn run() -> anyhow::Result<()> {
    let matches = app().get_matches();
    let mut leaf_nodes = Vec::new();
    let mut root = Rc::new(RefCell::new(Node::Root {
        values: Vec::new(),
        children: HashMap::new(),
    }));
    for file in matches.values_of("FILES").unwrap() {
        collect(file, load_file(file)?, &mut root, &mut leaf_nodes);
    }
    let mut combos: Vec<HashSet<String>> =
        leaf_nodes.iter().map(|n| n.borrow().present_in()).collect();
    combos.sort_by_key(|s| s.len());
    combos.dedup();
    combos.reverse();
    for combo in combos {
        for file in combo.iter() {
            println!("# {}", file);
        }
        println!(
            "{}",
            serde_yaml::to_string(&root.borrow().build_combo(&combo).unwrap()).unwrap()
        );
    }
    Ok(())
}

fn collect(
    file: &str,
    content: Value,
    parent: &mut Rc<RefCell<Node>>,
    leaf_nodes: &mut Vec<Rc<RefCell<Node>>>,
) {
    if let Value::Mapping(mapping) = content {
        for (k, v) in mapping.into_iter() {
            let key = k.as_str().unwrap();
            let mut node = parent.borrow_mut().add_intermediate_node(file, key);
            collect(file, v, &mut node, leaf_nodes);
        }
    } else {
        if let Some(leaf) = parent.borrow_mut().add_leaf_node(file, content) {
            leaf_nodes.push(leaf)
        }
    }
}

#[derive(Debug)]
enum Node {
    Root {
        children: HashMap<String, Rc<RefCell<Node>>>,
        values: Vec<Rc<RefCell<Node>>>,
    },
    Intermediate {
        present_in: HashSet<String>,
        children: HashMap<String, Rc<RefCell<Node>>>,
        values: Vec<Rc<RefCell<Node>>>,
    },
    Leaf {
        present_in: HashSet<String>,
        value: Value,
    },
}

impl Node {
    fn build_combo(&self, combo: &HashSet<String>) -> Option<Value> {
        if let Node::Leaf {
            value, present_in, ..
        } = self
        {
            return if present_in == combo {
                Some(value.clone())
            } else {
                None
            };
        }
        for leaf in self.values() {
            if let Some(value) = leaf.borrow().build_combo(&combo) {
                return Some(value);
            }
        }
        let mut result = Mapping::new();
        for (key, node) in self.children() {
            let node = node.borrow();
            if node.present_in().is_superset(combo) {
                if let Some(child) = node.build_combo(combo) {
                    result.insert(Value::String(key.clone()), child);
                }
            }
        }
        if result.len() > 0 {
            Some(Value::Mapping(result))
        } else {
            None
        }
    }

    fn add_intermediate_node(&mut self, file: &str, key: &str) -> Rc<RefCell<Node>> {
        let children = self.children_mut();
        if let Some(child) = children.get(key) {
            child.borrow_mut().add_present_in(file);
            child.clone()
        } else {
            let node = Rc::new(RefCell::new(Node::Intermediate {
                present_in: vec![file.to_string()].into_iter().collect(),
                children: HashMap::new(),
                values: Vec::new(),
            }));
            children.insert(key.to_string(), node.clone());
            node
        }
    }

    fn add_leaf_node(&mut self, file: &str, leaf_value: Value) -> Option<Rc<RefCell<Node>>> {
        let values = self.values_mut();
        if let Some(found) = values.iter().find(|c| {
            if let Some(value) = c.borrow().leaf_value() {
                value == &leaf_value
            } else {
                false
            }
        }) {
            found.borrow_mut().add_present_in(file);
            None
        } else {
            let node = Rc::new(RefCell::new(Node::Leaf {
                present_in: vec![file.to_string()].into_iter().collect(),
                value: leaf_value,
            }));
            values.push(node.clone());
            Some(node)
        }
    }

    fn values_mut(&mut self) -> &mut Vec<Rc<RefCell<Node>>> {
        match self {
            Self::Root { values, .. } => values,
            Self::Intermediate { values, .. } => values,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    fn values(&self) -> &Vec<Rc<RefCell<Node>>> {
        match self {
            Self::Root { values, .. } => values,
            Self::Intermediate { values, .. } => values,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    fn children_mut(&mut self) -> &mut HashMap<String, Rc<RefCell<Node>>> {
        match self {
            Self::Root { children, .. } => children,
            Self::Intermediate { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    fn children(&self) -> &HashMap<String, Rc<RefCell<Node>>> {
        match self {
            Self::Root { children, .. } => children,
            Self::Intermediate { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    fn leaf_value(&self) -> Option<&Value> {
        if let Self::Leaf { value, .. } = self {
            Some(&value)
        } else {
            None
        }
    }

    fn present_in(&self) -> HashSet<String> {
        match self {
            Self::Leaf { present_in, .. } => present_in.clone(),
            Self::Intermediate { present_in, .. } => present_in.clone(),
            _ => unreachable!(),
        }
    }

    fn add_present_in(&mut self, file: &str) {
        match self {
            Self::Leaf { present_in, .. } => {
                present_in.insert(file.to_string());
            }
            Self::Intermediate { present_in, .. } => {
                present_in.insert(file.to_string());
            }
            _ => unreachable!(),
        }
    }
}
