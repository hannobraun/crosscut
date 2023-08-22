use crate::language::{
    intrinsics,
    pipeline::{self, PipelineError, PipelineOutput},
    syntax::{Syntax, SyntaxToTokens},
    tokens::Tokens,
};

use super::{
    evaluator::{Evaluator, EvaluatorError, EvaluatorState},
    functions::Intrinsic,
    updater,
};

#[derive(Debug)]
pub struct Interpreter {
    pub tokens: Tokens,
    pub syntax: Syntax,
    pub syntax_to_tokens: SyntaxToTokens,
    pub evaluator: Evaluator,
}

impl Interpreter {
    pub fn new(code: &str) -> Result<Self, PipelineError> {
        let mut syntax = Syntax::new();
        let PipelineOutput {
            start,
            tokens,
            syntax_to_tokens,
        } = pipeline::run(code, &mut syntax)?;

        let mut evaluator = Evaluator::new();
        if let Some(start) = start {
            evaluator.call_stack.push(start);
        }

        let intrinsics = [
            ("+", intrinsics::add as Intrinsic),
            ("clone", intrinsics::clone),
            ("delay_ms", intrinsics::delay_ms),
            ("print_line", intrinsics::print_line),
            ("fn", intrinsics::fn_),
        ];

        for (name, intrinsic) in intrinsics {
            evaluator.functions.register_intrinsic(name, intrinsic)
        }

        Ok(Interpreter {
            tokens,
            syntax,
            syntax_to_tokens,
            evaluator,
        })
    }

    pub fn step(&mut self) -> Result<EvaluatorState, EvaluatorError> {
        self.evaluator.step(&self.syntax)
    }

    pub fn update(&mut self, code: &str) -> Result<(), PipelineError> {
        self.syntax.prepare_update();
        let PipelineOutput {
            tokens,
            syntax_to_tokens,
            ..
        } = pipeline::run(code, &mut self.syntax)?;
        updater::update(
            &self.tokens,
            &tokens,
            &self.syntax,
            &self.syntax_to_tokens,
            &mut self.evaluator,
        );

        self.tokens = tokens;
        self.syntax_to_tokens = syntax_to_tokens;

        Ok(())
    }
}
