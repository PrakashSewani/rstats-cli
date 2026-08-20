use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct History {
    values: VecDeque<f64>,
    capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self { values: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, value: f64) {
        if self.capacity == 0 {
            return;
        }
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub fn values(&self) -> impl Iterator<Item = &f64> {
        self.values.iter()
    }

    pub fn as_vec(&self) -> Vec<f64> {
        self.values.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_newest_values() {
        let mut history = History::new(2);
        history.push(1.0);
        history.push(2.0);
        history.push(3.0);
        assert_eq!(history.as_vec(), vec![2.0, 3.0]);
    }
}
