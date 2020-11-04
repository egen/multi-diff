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
    leaf_nodes.sort_by_key(|n| n.borrow().present_in());
    println!("root:\n{:?}", root);
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
            let mut node = parent.borrow_mut().add_intermediate_node(key, &parent);
            collect(file, v, &mut node, leaf_nodes);
        }
    } else {
        if let Some(leaf) = parent.borrow_mut().add_leaf_node(file, content, &parent) {
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
        children: HashMap<String, Rc<RefCell<Node>>>,
        values: Vec<Rc<RefCell<Node>>>,
        key: String,
        parent: Rc<RefCell<Node>>,
    },
    Leaf {
        present_in: HashSet<String>,
        parent: Rc<RefCell<Node>>,
        value: Value,
    },
}
impl Node {
    fn add_intermediate_node(
        &mut self,
        key: &str,
        parent: &Rc<RefCell<Node>>,
    ) -> Rc<RefCell<Node>> {
        let children = self.children_mut();
        if let Some(child) = children.get(key) {
            child.clone()
        } else {
            let node = Rc::new(RefCell::new(Node::Intermediate {
                children: HashMap::new(),
                values: Vec::new(),
                key: key.to_string(),
                parent: parent.clone(),
            }));
            children.insert(key.to_string(), node.clone());
            node
        }
    }

    fn add_leaf_node(
        &mut self,
        file: &str,
        leaf_value: Value,
        parent: &Rc<RefCell<Node>>,
    ) -> Option<Rc<RefCell<Node>>> {
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
                parent: parent.clone(),
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

    fn children_mut(&mut self) -> &mut HashMap<String, Rc<RefCell<Node>>> {
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

    fn present_in(&self) -> usize {
        if let Self::Leaf { present_in, .. } = self {
            present_in.len()
        } else {
            unreachable!()
        }
    }

    fn add_present_in(&mut self, file: &str) {
        if let Self::Leaf { present_in, .. } = self {
            present_in.insert(file.to_string());
        } else {
            unreachable!()
        }
    }
}
