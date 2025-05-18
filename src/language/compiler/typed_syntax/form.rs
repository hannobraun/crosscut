pub trait Form {
    type Form<T>;
}

pub struct Owned;

impl Form for Owned {
    type Form<T> = T;
}
