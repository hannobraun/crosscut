use crate::code::Binding;

use super::{located::HasLocation, ParameterLocation};

impl HasLocation for Binding {
    type Location = ParameterLocation;
}
