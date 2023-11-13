use std::ptr::NonNull;

#[derive(Clone)]
pub struct GcPtr<T>(NonNull<T>);

impl GcPtr<Object> {
    unsafe fn mark(&mut self) {
        if self.0.as_ref().marked {
            return;
        }

        self.0.as_mut().marked = true;

        if let ObjType::Pair(pair) = &mut self.0.as_mut().value {
            if let Some(ref mut head) = &mut pair.head {
                head.mark();
            }
            if let Some(ref mut tail) = &mut pair.tail {
                tail.mark();
            }
        }
    }

    fn is_marked(&self) -> bool {
        unsafe { self.0.as_ref().marked }
    }
}

#[derive(Clone)]
pub struct Object {
    marked: bool,
    value: ObjType,
}

#[derive(Clone)]
pub enum ObjType {
    Int(i64),
    Pair(Pair),
}

#[derive(Clone)]
pub struct Pair {
    head: Option<GcPtr<Object>>,
    tail: Option<GcPtr<Object>>,
}

const STACK_MAX: usize = 256;

pub struct Vm {
    stack: [Option<GcPtr<Object>>; STACK_MAX],
    stack_size: usize,
    heap: Vec<GcPtr<Object>>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: std::array::from_fn(|_| None),
            stack_size: 0,
            heap: vec![],
        }
    }

    pub fn push(&mut self, value: ObjType) {
        assert!(self.stack_size < STACK_MAX, "Stack overflow!");
        let mut box_obj = Box::new(Object {
            marked: false,
            value,
        });
        let gc_ptr = GcPtr(NonNull::new(&mut *box_obj).unwrap());
        self.stack[self.stack_size] = Some(gc_ptr.clone());
        std::mem::forget(box_obj);
        self.heap.push(gc_ptr);
        self.stack_size += 1;
    }

    pub fn pop(&mut self) -> GcPtr<Object> {
        let obj = self.stack[self.stack_size].take().unwrap();
        self.stack_size -= 1;
        obj
    }

    pub fn push_int(&mut self, value: i64) {
        self.push(ObjType::Int(value));
    }

    pub fn push_pair(&mut self) {
        let head = Some(self.pop());
        let tail = Some(self.pop());
        self.push(ObjType::Pair(Pair { head, tail }));
    }

    pub fn mark_all(&mut self) {
        for obj in &mut self.stack {
            if let Some(obj) = obj {
                unsafe {
                    obj.mark();
                }
            }
        }
    }

    pub fn sweep(&mut self) {
        let mut live_objects = vec![];

        for obj in &mut self.heap {
            if !obj.is_marked() {
                unsafe {
                    let unreached = obj.0.as_mut();
                    let _ = Box::from_raw(unreached); // drop
                }
            } else {
                unsafe {
                    obj.0.as_mut().marked = false;
                }
                live_objects.push(obj.clone()); // ptr clone
            }
        }

        self.heap = live_objects;
    }

    pub fn gc(&mut self) {
        self.mark_all();
        self.sweep();
    }
}
