use std::cell::Cell;
use std::rc::Rc;

pub struct DropMock {
    counter: Rc<Cell<usize>>,
}

impl DropMock {
    pub fn new() -> DropMock {
        DropMock {
            counter: Rc::new(Cell::new(0)),
        }
    }

    pub fn drop_cnt(&self) -> usize {
        self.counter.get()
    }
}

impl Clone for DropMock {
    fn clone(&self) -> Self {
        DropMock {
            counter: self.counter.clone(),
        }
    }
}

impl Drop for DropMock {
    fn drop(&mut self) {
        self.counter.set(self.counter.get() + 1);
    }
}
