use std::ptr::NonNull;

pub struct GcPtr<T>(NonNull<T>);

pub struct Object {
    marked: bool,
    typ: ObjType,
}

pub enum ObjType {
    Int(i64),
    Pair(Pair),
}

pub struct Pair {
    head: Option<GcPtr<Object>>,
    tail: Option<GcPtr<Object>>,
}
