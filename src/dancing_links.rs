use std::ops;

#[derive(Debug, Clone)]
pub struct DoublyLinkedList<T> {
    pub data: Vec<Link<T>>,
}

#[derive(Debug, Clone)]
pub struct Link<T> {
    pub prev: T,
    pub next: T,
}
pub trait Indexed {
    fn get_index(&self) -> usize;
    fn set_index(&mut self, index: usize);
}

impl<T: Indexed> ops::Index<T> for DoublyLinkedList<T> {
    type Output = Link<T>;
    fn index(&self, indexed: T) -> &Link<T> {
        &self.data[indexed.get_index()]
    }
}

impl<T: Indexed> ops::IndexMut<T> for DoublyLinkedList<T> {
    fn index_mut(&mut self, indexed: T) -> &mut Link<T> {
        &mut self.data[indexed.get_index()]
    }
}

pub struct Cursor<T> {
    head: T,
    curr: T,
}

impl<T: Indexed + Copy + Eq> Cursor<T> {
    pub fn next(&mut self, list: &DoublyLinkedList<T>) -> Option<T> {
        self.curr = list[self.curr].next;

        if self.curr == self.head {
            return None;
        }
        Some(self.curr)
    }
    pub fn prev(&mut self, list: &DoublyLinkedList<T>) -> Option<T> {
        self.curr = list[self.curr].prev;
        if self.curr == self.head {
            return None;
        }
        Some(self.curr)
    }
}

impl<T: Indexed + Default + Copy + std::fmt::Debug> DoublyLinkedList<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
        }
    }

    pub fn alloc(&mut self) -> T {
        let mut element = T::default();
        element.set_index(self.data.len());
        self.data.push(Link {
            prev: element,
            next: element,
        });
        element
    }

    pub fn insert(&mut self, a: T, b: T) {
        let c = self[a].next;

        self[b].prev = a;
        self[b].next = c;

        self[c].prev = b;
        self[a].next = b;
    }

    pub fn remove(&mut self, b: T) {
        let a = self[b].prev;
        let c = self[b].next;

        self[a].next = c;
        self[c].prev = a;
    }

    pub fn restore(&mut self, b: T) {
        let a = self[b].prev;
        let c = self[b].next;

        self[a].next = b;
        self[c].prev = b;
    }

    pub fn iter(&self, head: T) -> Cursor<T> {
        Cursor { head, curr: head }
    }
}
