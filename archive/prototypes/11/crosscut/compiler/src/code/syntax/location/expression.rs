use crate::code::syntax::{Expression, Function};

use super::{
    located::HasLocation, member::MemberLocation, FunctionLocation, Located,
};

impl HasLocation for Expression {
    type Location = MemberLocation;
}

impl<'r> Located<&'r Expression> {
    /// # Convert the located expression into a located local function
    pub fn into_local_function(self) -> Option<Located<&'r Function>> {
        self.fragment.as_local_function().map(|function| Located {
            fragment: function,
            location: FunctionLocation::Local {
                location: self.location.clone(),
            },
        })
    }
}
