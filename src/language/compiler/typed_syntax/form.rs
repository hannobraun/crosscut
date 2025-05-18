use std::marker::PhantomData;

use crate::language::code::NodeHash;

pub trait Form {
    type Form<T: 'static>;
}

pub struct NodeRef<'r>(PhantomData<&'r ()>);

impl<'r> Form for NodeRef<'r> {
    type Form<T: 'static> = &'r NodeHash;
}

pub struct Owned;

impl Form for Owned {
    type Form<T: 'static> = T;
}

pub struct Ref<'r>(PhantomData<&'r ()>);

impl<'r> Form for Ref<'r> {
    type Form<T: 'static> = &'r T;
}
