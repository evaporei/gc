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

    fn unmark(&mut self) {
        unsafe {
            self.0.as_mut().marked = false;
        }
    }

    unsafe fn free(&mut self) {
        let unreached = self.0.as_mut();
        let _ = Box::from_raw(unreached); // drop
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
const INITIAL_GC_THRESHOLD: usize = 8;

pub struct Vm {
    stack: [Option<GcPtr<Object>>; STACK_MAX],
    stack_size: usize,
    heap: Vec<GcPtr<Object>>,
    /// currently total number of objects allocated
    num_objs: usize,
    /// number of objects required to trigger a GC
    max_objs: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: std::array::from_fn(|_| None),
            stack_size: 0,
            heap: vec![],
            num_objs: 0,
            max_objs: INITIAL_GC_THRESHOLD,
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
        self.num_objs += 1;
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
                unsafe { obj.free() }
                self.num_objs -= 1;
            } else {
                obj.unmark();
                live_objects.push(obj.clone()); // ptr clone
            }
        }

        self.heap = live_objects;
    }

    pub fn gc(&mut self) {
        let num_objs = self.num_objs;

        self.mark_all();
        self.sweep();

        self.max_objs = if self.num_objs == 0 {
            INITIAL_GC_THRESHOLD
        } else {
            self.num_objs * 2
        };

        println!("Collected {} objects, {} remaining.", num_objs - self.num_objs, self.num_objs);
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.stack_size = 0;
        self.stack = std::array::from_fn(|_| None);
        self.gc();
    }
}

#[test]
fn test1() {
    println!("Test 1: Objects on stack are preserved.");
    let mut vm = Vm::new();
    vm.push_int(1);
    vm.push_int(2);

    vm.gc();
    assert!(vm.num_objs == 2, "Should have preserved objects.");
    drop(vm);
}
