use std::{cell::RefCell, rc::Rc};

enum ObjectType {
    Int(usize),
    Pair(Pair),
}

struct Pair {
    head: Rc<RefCell<Object>>,
    tail: Rc<RefCell<Object>>,
}

struct Object {
    obj_type: ObjectType,
    marked: bool,
    next: Option<Rc<RefCell<Object>>>,
}

struct VM {
    stack: Vec<Rc<RefCell<Object>>>,
    max_size: usize,
    first_object: Option<Rc<RefCell<Object>>>,
    max_objects: usize,
    num_objects: usize,
}

impl VM {
    pub fn new(max_size: usize) -> Self {
        VM {
            stack: Vec::with_capacity(max_size),
            max_size,
            first_object: None,
            max_objects: 8,
            num_objects: 0,
        }
    }

    pub fn set_pair_tail(obj: Rc<RefCell<Object>>, new_tail: Rc<RefCell<Object>>) {
        match &mut obj.borrow_mut().obj_type {
            ObjectType::Pair(ref mut pair) => {
                pair.tail = new_tail;
            }
            _ => panic!("should be a pair"),
        }
    }

    pub fn push_int(&mut self, value: usize) -> Rc<RefCell<Object>> {
        self.new_object(ObjectType::Int(value))
    }

    pub fn push_pair(&mut self) -> Rc<RefCell<Object>> {
        let tail = self.pop();
        let head = self.pop();
        self.new_object(ObjectType::Pair(Pair { head, tail }))
    }

    pub fn gc(&mut self) {
        let num_objects = self.num_objects;

        self.mark_all();
        self.sweep();

        self.max_objects = self.num_objects * 2;

        println!(
            "Collected {} objects, {} remaining.",
            num_objects - self.num_objects,
            self.num_objects
        );
    }

    fn mark(obj: Rc<RefCell<Object>>) {
        if obj.borrow().marked {
            return;
        }

        obj.borrow_mut().marked = true;

        match &obj.borrow().obj_type {
            ObjectType::Int(_) => {}
            ObjectType::Pair(pair) => {
                VM::mark(pair.head.clone());
                VM::mark(pair.tail.clone());
            }
        }
    }

    fn push(&mut self, obj: Rc<RefCell<Object>>) {
        if self.stack.len() >= self.max_size {
            panic!("Stack overflow");
        }
        self.stack.push(obj);
    }

    fn pop(&mut self) -> Rc<RefCell<Object>> {
        if self.stack.is_empty() {
            panic!("Stack underflow");
        }

        self.stack.pop().unwrap()
    }

    fn new_object(&mut self, obj_type: ObjectType) -> Rc<RefCell<Object>> {
        if self.num_objects >= self.max_objects {
            self.gc();
        }

        let obj = Rc::new(RefCell::new(Object {
            obj_type,
            marked: false,
            next: self.first_object.clone(),
        }));

        self.push(obj.clone());
        self.num_objects += 1;
        self.first_object = Some(obj.clone());
        obj
    }

    fn mark_all(&mut self) {
        for obj in self.stack.iter_mut() {
            VM::mark(obj.clone());
        }
    }

    fn sweep(&mut self) {
        let mut obj = self.first_object.clone();

        while let Some(o) = obj {
            if !o.borrow().marked {
                let unreached = o.clone();

                obj = unreached.borrow().next.clone();

                self.num_objects -= 1;

                drop(unreached);
            } else {
                o.borrow_mut().marked = false;
                obj = o.borrow().next.clone();
            }
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_objects_are_preserved() {
        let mut vm = VM::new(10);

        vm.push_int(1);
        vm.push_int(2);

        vm.gc();

        assert_eq!(vm.num_objects, 2);
    }

    #[test]
    fn unreached_objects_are_collected() {
        let mut vm = VM::new(10);

        vm.push_int(1);
        vm.push_int(2);

        vm.pop();
        vm.pop();

        vm.gc();

        assert_eq!(vm.num_objects, 0);
    }

    #[test]
    fn nested_objects_are_reachable() {
        let mut vm = VM::new(10);

        vm.push_int(1);
        vm.push_int(2);
        vm.push_pair();
        vm.push_int(3);
        vm.push_int(4);
        vm.push_pair();
        vm.push_pair();

        vm.gc();

        assert_eq!(vm.num_objects, 7);
    }

    #[test]
    fn handles_cycles() {
        let mut vm = VM::new(10);

        vm.push_int(1);
        vm.push_int(2);
        let a = vm.push_pair();
        vm.push_int(3);
        vm.push_int(4);
        let b = vm.push_pair();

        VM::set_pair_tail(a.clone(), b.clone());
        VM::set_pair_tail(b, a.clone());

        vm.gc();

        assert_eq!(vm.num_objects, 4);
    }
}
