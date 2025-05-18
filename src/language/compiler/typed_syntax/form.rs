use std::marker::PhantomData;

use crate::language::code::NodeHash;

pub trait Form {
    type Form<T: 'static>;
}

pub struct NodeByHash;

impl Form for NodeByHash {
    type Form<T: 'static> = NodeHash;
}

pub struct Owned;

impl Form for Owned {
    type Form<T: 'static> = T;
}

pub struct Ref<'r>(PhantomData<&'r ()>);

impl<'r> Form for Ref<'r> {
    type Form<T: 'static> = &'r T;
}

pub struct RefMut<'r>(PhantomData<&'r mut ()>);

impl<'r> Form for RefMut<'r> {
    type Form<T: 'static> = &'r mut T;
}
