use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct UnionFind<T> {
    parent: HashMap<T, T>,
}

impl<T: Eq + Hash + Clone> UnionFind<T> {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    fn find(&mut self, x: T) -> T {
        let parent = self.parent.get(&x).cloned().unwrap_or(x.clone());

        if parent != x {
            let root = self.find(parent);
            self.parent.insert(x, root.clone());
            root
        } else {
            x
        }
    }

    pub fn union(&mut self, a: T, b: T) {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a != root_b {
            self.parent.insert(root_a, root_b);
        }
    }

    pub fn canonical(&mut self, x: T) -> T {
        self.find(x)
    }
}

impl<T: Eq + Hash + Clone> Default for UnionFind<T> {
    fn default() -> Self {
        Self::new()
    }
}
