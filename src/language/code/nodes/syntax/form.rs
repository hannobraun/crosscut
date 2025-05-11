use std::marker::PhantomData;

use crate::language::code::NodeHash;

pub trait Form {
    type Form<T>
    where
        T: 'static;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Borrowed<'r>(PhantomData<&'r ()>);

impl<'r> Form for Borrowed<'r> {
    type Form<T>
        = &'r T
    where
        T: 'static;
}

pub struct ViaHash;

impl Form for ViaHash {
    type Form<T>
        = NodeHash
    where
        T: 'static;
}
