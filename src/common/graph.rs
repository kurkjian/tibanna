use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// An undirected, bidirectional graph implemented using an adjacency list
#[derive(Debug, Default, Clone)]
pub struct Graph<T> {
    g: HashMap<T, HashSet<T>>,
}

impl<T: Eq + Hash + Clone> Graph<T> {
    pub fn new() -> Self {
        Self { g: HashMap::new() }
    }

    pub fn add_vertex(&mut self, vertex: T) {
        self.g.entry(vertex).or_insert_with(|| HashSet::new());
    }

    pub fn add_edge(&mut self, vertex1: T, vertex2: T) {
        if vertex1 == vertex2 {
            return;
        }
        self.g
            .entry(vertex1.clone())
            .or_insert_with(|| HashSet::new())
            .insert(vertex2.clone());
        self.g
            .entry(vertex2)
            .or_insert_with(|| HashSet::new())
            .insert(vertex1);
    }

    pub fn remove_vertex(&mut self, vertex: &T) -> Option<HashSet<T>> {
        let neighbors = self.g.remove(vertex)?;
        for (_, neighbors) in self.g.iter_mut() {
            neighbors.remove(vertex);
        }
        Some(neighbors)
    }

    pub fn is_empty(&self) -> bool {
        self.g.is_empty()
    }

    pub fn len(&self) -> usize {
        self.g.len()
    }

    pub fn vertices(&self) -> impl Iterator<Item = &T> {
        self.g.keys()
    }

    pub fn degree(&self, vertex: &T) -> usize {
        self.g.get(vertex).map(|s| s.len()).unwrap_or(0)
    }

    pub fn edges(&self, vertex: &T) -> Option<&HashSet<T>> {
        self.g.get(vertex)
    }
}
