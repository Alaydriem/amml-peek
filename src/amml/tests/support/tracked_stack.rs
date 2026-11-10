use std::cell::Cell;
use std::rc::Rc;

// a stack that counts its drops in a counter shared with the test
pub struct TrackedStack {
    pub name: String,
    drops: Rc<Cell<usize>>,
}

impl TrackedStack {
    pub fn new(name: &str, drops: &Rc<Cell<usize>>) -> Self {
        Self { name: name.to_string(), drops: Rc::clone(drops) }
    }
}

impl Drop for TrackedStack {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
