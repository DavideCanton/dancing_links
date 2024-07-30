use std::cell::RefCell;

use priority_queue::PriorityQueue;

use crate::cells::HeaderRef;

pub(crate) struct HeaderPriorityQueue<'a, T> {
    queue: RefCell<PriorityQueue<HeaderRef<'a, T>, (isize, usize)>>,
}

impl<'a, T> HeaderPriorityQueue<'a, T> {
    pub(crate) fn new() -> Self {
        HeaderPriorityQueue {
            queue: RefCell::new(PriorityQueue::new()),
        }
    }

    pub(crate) fn push(&self, header: HeaderRef<'a, T>) {
        self.queue
            .borrow_mut()
            .push(header, header_priority(header));
    }

    pub(crate) fn remove(&self, header: HeaderRef<'a, T>) {
        self.queue.borrow_mut().remove(&header);
    }

    pub(crate) fn change_priority(&self, header: HeaderRef<'a, T>) {
        self.queue
            .borrow_mut()
            .change_priority(&header, header_priority(header));
    }

    pub(crate) fn peek(&self) -> Option<HeaderRef<'a, T>> {
        self.queue.borrow().peek().map(|(header, _)| *header)
    }
}

fn header_priority<'a, T>(header: HeaderRef<'a, T>) -> (isize, usize) {
    (-(header.size() as isize), header.index)
}
