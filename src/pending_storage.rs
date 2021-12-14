use std::collections::HashSet;

pub struct PendingStorage {
    pending_confirmation: HashSet<String>,
}

impl Default for PendingStorage {
    fn default() -> Self {
        Self::new()
    }
}


impl PendingStorage {
    pub fn new() -> PendingStorage {
        PendingStorage {
            pending_confirmation: HashSet::new(),
        }
    }

    pub fn store(&mut self, value: String) {
        self.pending_confirmation.insert(value);
    }

    pub fn remove(&mut self, value: String) {
        self.pending_confirmation.remove(&value);
    }

    pub fn get_all(&self) -> Vec<String> {
        let mut res = vec![];
        for v in &self.pending_confirmation {
            res.push(v.clone());
        }
        res
    }
}
