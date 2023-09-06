use crate::language::repr::eval::fragments::{Fragments, Replacement};

use super::evaluator::Evaluator;

pub fn update(fragments: &mut Fragments, evaluator: &mut Evaluator) {
    for Replacement { old, new } in fragments.take_replacements() {
        evaluator.functions.replace(old, new);
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use crate::language::{
        repr::eval::fragments::FragmentId,
        runtime::{
            functions::{self, Function},
            interpreter::Interpreter,
        },
    };

    #[test]
    fn update_at_beginning_of_named_function() -> anyhow::Result<()> {
        let original = ":f { 1 + } fn";
        let updated = ":f { 2 + } fn";

        let mut interpreter = Interpreter::new(original)?;
        while interpreter.step()?.in_progress() {}

        let f_original = extract("f", &interpreter)?;

        interpreter.update(updated)?;
        let f_updated = extract("f", &interpreter)?;

        assert_ne!(f_original, f_updated);

        Ok(())
    }

    #[test]
    fn update_that_reverts_back_to_an_earlier_version() -> anyhow::Result<()> {
        let original = ":f { 1 + } fn";
        let updated = ":f { 2 + } fn";

        let mut interpreter = Interpreter::new(original)?;
        while interpreter.step()?.in_progress() {}

        interpreter.update(updated)?;
        let f_updated = extract("f", &interpreter)?;

        interpreter.update(original)?;
        let f_original = extract("f", &interpreter)?;

        assert_ne!(f_updated, f_original);

        Ok(())
    }

    #[test]
    fn update_in_middle_of_named_function() -> anyhow::Result<()> {
        let original = ":f { 1 1 + } fn";
        let updated = ":f { 1 2 + } fn";

        let mut interpreter = Interpreter::new(original)?;
        while interpreter.step()?.in_progress() {}

        let f_original = extract("f", &interpreter)?;

        interpreter.update(updated)?;
        let f_updated = extract("f", &interpreter)?;

        assert_ne!(f_original, f_updated);

        Ok(())
    }

    fn extract(
        name: &str,
        interpreter: &Interpreter,
    ) -> anyhow::Result<FragmentId> {
        let function = interpreter.evaluator.functions.resolve(name)?;

        let Function::UserDefined(functions::UserDefined { body }) = function
        else {
            bail!("Expected function `f` to be user-defined")
        };

        let Some(id) = body.start else {
            bail!("Expected function `f` to not be empty")
        };

        Ok(id)
    }
}
