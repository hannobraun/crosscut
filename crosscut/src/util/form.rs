use std::marker::PhantomData;

pub trait Form {
    type Form<T: 'static>;
}

#[derive(Debug)]
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
