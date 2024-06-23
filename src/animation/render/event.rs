use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub struct EventSender<E> {
    pool: Rc<RefCell<VecDeque<E>>>,
}

impl<E> Clone for EventSender<E> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl<E> EventSender<E> {
    pub fn add_event(&self, event: E) {
        self.pool.borrow_mut().push_back(event);
    }
}

pub struct EventReciever<E> {
    pool: Rc<RefCell<VecDeque<E>>>,
}

impl<E> EventReciever<E> {
    pub fn new() -> Self {
        Self {
            pool: Rc::new(RefCell::new(VecDeque::new()))
        }
    }

    pub fn create_sender(&self) -> EventSender<E> {
        EventSender {
            pool: self.pool.clone()
        }
    }

    pub fn poll_event(&self) -> Option<E> {
        self.pool.borrow_mut().pop_front()
    }
}
