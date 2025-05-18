use std::marker::PhantomData;

use crate::language::code::NodeHash;

pub trait Form {
    type Form<T>;
}

pub struct Owned;

impl Form for Owned {
    type Form<T> = T;
}

pub struct Ref<'r>(PhantomData<&'r ()>);

impl<'r> Form for Ref<'r> {
    type Form<T> = &'r NodeHash;
}
