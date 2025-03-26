use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {}

pub struct ExpressionDisplay {}

impl fmt::Display for ExpressionDisplay {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
