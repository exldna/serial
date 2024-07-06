use std::marker::PhantomData;
use std::ptr::NonNull;

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    next: Link<T>,
    prev: Link<T>,
    element: T,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Self {
            next: None,
            prev: None,
            element,
        }
    }

    fn into_element(self: Box<Self>) -> T {
        self.element
    }
}

pub struct LinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _pd: PhantomData<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _pd: PhantomData,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LinkedList<T> {
    pub fn front(&self) -> Option<&T> {
        unsafe { 
            self.head.as_ref().map(|node| &node.as_ref().element) 
        }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { 
            self.head.as_mut().map(|node| &mut node.as_mut().element) 
        }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe {
            self.tail.as_ref().map(|node| &node.as_ref().element)
        }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.tail.as_mut().map(|node| &mut node.as_mut().element)
        }
    }
    
    pub fn push_front(&mut self, element: T) {
        let node = Box::new(Node::new(element));
        let node_ptr = NonNull::from(Box::leak(node));
        // SAFETY: node_ptr is a unique pointer to a node we boxed and leaked
        unsafe {
            self.push_front_node(node_ptr);
        }
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        self.pop_front_node().map(Node::into_element)
    }

    pub fn push_back(&mut self, element: T) {
        let node = Box::new(Node::new(element));
        let node_ptr = NonNull::from(Box::leak(node));
        // SAFETY: node_ptr is a unique pointer to a node we boxed and leaked
        unsafe {
            self.push_back_node(node_ptr);
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.pop_back_node().map(Node::into_element)
    }
}

// Unsafe internals (from std 🙂)
impl<T> LinkedList<T> {
    /// Adds the given node to the front of the list.
    ///
    /// # Safety
    /// `node` must point to a valid node that was boxed and leaked using the list's allocator.
    /// This method takes ownership of the node, so the pointer should not be used again.
    unsafe fn push_front_node(&mut self, node: NonNull<Node<T>>) {
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        unsafe {
            (*node.as_ptr()).next = self.head;
            (*node.as_ptr()).prev = None;
            let node = Some(node);

            match self.head {
                None => self.tail = node,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(head) => (*head.as_ptr()).prev = node,
            }

            self.head = node;
            self.len += 1;
        }
    }

    /// Removes and returns the node at the front of the list.
    fn pop_front_node(&mut self) -> Option<Box<Node<T>>> {
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        self.head.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.head = node.next;

            match self.head {
                None => self.tail = None,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(head) => (*head.as_ptr()).prev = None,
            }

            self.len -= 1;
            node
        })
    }

    /// Adds the given node to the back of the list.
    ///
    /// # Safety
    /// `node` must point to a valid node that was boxed and leaked using the list's allocator.
    /// This method takes ownership of the node, so the pointer should not be used again.
    unsafe fn push_back_node(&mut self, node: NonNull<Node<T>>) {
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        unsafe {
            (*node.as_ptr()).next = None;
            (*node.as_ptr()).prev = self.tail;
            let node = Some(node);

            match self.tail {
                None => self.head = node,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(tail) => (*tail.as_ptr()).next = node,
            }

            self.tail = node;
            self.len += 1;
        }
    }

    /// Removes and returns the node at the back of the list.
    fn pop_back_node(&mut self) -> Option<Box<Node<T>>> {
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        self.tail.map(|node| unsafe {
            let node = Box::from_raw(node.as_ptr());
            self.tail = node.prev;

            match self.tail {
                None => self.head = None,
                // Not creating new mutable (unique!) references overlapping `element`.
                Some(tail) => (*tail.as_ptr()).next = None,
            }

            self.len -= 1;
            node
        })
    }
}

unsafe impl<#[may_dangle] T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front_node().is_some() {}
    }
}
