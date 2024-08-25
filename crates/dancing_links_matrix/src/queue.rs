use std::cell::RefCell;

use priority_queue::PriorityQueue;

use crate::cells::{ColumnInfo, ColumnRef};

type PriorityKey = (isize, usize);

pub(crate) struct ColumnPriorityQueue<'a, T> {
    queue: RefCell<PriorityQueue<ColumnRef<'a, T>, PriorityKey>>,
}

impl<'a, T> ColumnPriorityQueue<'a, T> {
    pub(crate) fn new() -> Self {
        ColumnPriorityQueue {
            queue: RefCell::new(PriorityQueue::new()),
        }
    }

    pub(crate) fn push(&self, column: ColumnRef<'a, T>) {
        let priority = column_priority(column);
        let mut queue = self.queue.borrow_mut();
        queue.push(column, priority);
    }

    pub(crate) fn remove(&self, column: ColumnRef<'a, T>) {
        self.queue.borrow_mut().remove(&column);
    }

    pub(crate) fn change_priority(&self, column: ColumnRef<'a, T>) {
        self.queue
            .borrow_mut()
            .change_priority(&column, column_priority(column));
    }

    pub(crate) fn peek(&self) -> Option<ColumnRef<'a, T>> {
        self.queue.borrow().peek().map(|(column, _)| *column)
    }
}

fn column_priority<T>(column: &ColumnInfo<'_, T>) -> PriorityKey {
    (-(column.size() as isize), column.index)
}
