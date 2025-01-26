use crosscut_compiler::{
    code::{
        syntax::{MemberLocation, SyntaxTree},
        FunctionCalls,
    },
    Compiler,
};
use crosscut_game_engine::{
    command::Command, game_engine::GameEngine, host::GameEngineHost,
    memory::Memory,
};
use crosscut_protocol::updates::Updates;

use crate::model::{
    ActiveFunctions, ActiveFunctionsEntry, DebugBranch, DebugFunction,
    DebugMember, DebugMemberKind, DebugNamedFunction, PersistentState,
    TransientState, UserAction,
};

pub fn debugger() -> TestDebugger {
    TestDebugger::default()
}

#[derive(Debug, Default)]
pub struct TestDebugger {
    current_time: f64,
    queued_commands: Vec<Command>,
    memory: Memory,
    updates: Updates,
    game_engine: Option<GameEngine>,
    persistent: PersistentState,
    transient: Option<TransientState>,
}

impl TestDebugger {
    pub fn provide_source_code(&mut self, source: &str) -> &mut Self {
        let mut compiler = Compiler::default();
        let output = compiler.compile(source, &GameEngineHost);

        let command = self.persistent.on_new_code(output);
        self.queued_commands.push(command);

        self.update_transient_state();

        self
    }

    pub fn run_program(&mut self) -> &mut Self {
        self.game_engine = Some(GameEngine::new());

        self.process_commands();
        self.process_updates();
        self.update_transient_state();

        self
    }

    pub fn on_user_action(
        &mut self,
        action: UserAction,
    ) -> anyhow::Result<&mut Self> {
        let transient = self.transient_state();
        let commands = self.persistent.on_user_action(action, &transient)?;

        for command in commands {
            self.queued_commands.push(command);
        }

        self.process_commands();
        self.process_updates();
        self.update_transient_state();

        Ok(self)
    }

    fn process_commands(&mut self) {
        if let Some(game_engine) = &mut self.game_engine {
            for command in self.queued_commands.drain(..) {
                game_engine.on_command(command);
            }

            let mut pixels = [];
            game_engine.run_until_end_of_frame(self.current_time, &mut pixels);
            self.current_time += 1.;
        }
    }

    fn process_updates(&mut self) {
        if let Some(game_engine) = &self.game_engine {
            self.updates
                .queue_updates(&game_engine.runtime, &self.memory);
            for update in self.updates.take_queued_updates() {
                self.persistent.on_update_from_host(update);
            }
        }
    }

    fn update_transient_state(&mut self) {
        self.transient = Some(self.persistent.generate_transient_state());
    }

    pub fn expect_code(&self) -> &SyntaxTree {
        &self.persistent.code.inner.as_ref().unwrap().syntax_tree
    }

    pub fn expect_expression(
        &mut self,
        location: &MemberLocation,
    ) -> DebugMember {
        let Some(expression) = self
            .transient_state()
            .active_functions
            .expect_entries()
            .expect_functions()
            .into_iter()
            .find_map(|function| {
                function.inner.branches.iter().find_map(|branch| {
                    branch.body.iter().find_map(|expression| {
                        if &expression.data.location == location {
                            Some(expression.clone())
                        } else {
                            None
                        }
                    })
                })
            })
        else {
            panic!("Expected to find expression with location `{location:?}`");
        };

        expression
    }

    pub fn persistent_state(&self) -> &PersistentState {
        &self.persistent
    }

    pub fn transient_state(&mut self) -> TransientState {
        self.transient
            .get_or_insert_with(|| self.persistent.generate_transient_state())
            .clone()
    }
}

pub trait ActiveFunctionsExt {
    fn expect_entries(&self) -> Vec<ActiveFunctionsEntry>;
    fn names(&self) -> Vec<String>;
}

impl ActiveFunctionsExt for ActiveFunctions {
    fn expect_entries(&self) -> Vec<ActiveFunctionsEntry> {
        self.entries().unwrap().inner.to_vec()
    }

    fn names(&self) -> Vec<String> {
        self.expect_entries()
            .expect_functions()
            .into_iter()
            .map(|function| function.name)
            .collect()
    }
}

pub trait ActiveFunctionsEntriesExt {
    fn expect_functions(&self) -> Vec<DebugNamedFunction>;
}

impl ActiveFunctionsEntriesExt for Vec<ActiveFunctionsEntry> {
    fn expect_functions(&self) -> Vec<DebugNamedFunction> {
        self.iter()
            .map(|entry| match entry {
                ActiveFunctionsEntry::Function(function) => function.clone(),
                ActiveFunctionsEntry::Gap => {
                    panic!(
                        "Expected function, encountered gap. Entries:\n\
                        {self:#?}"
                    );
                }
            })
            .collect()
    }
}

pub trait FunctionsExt {
    fn expect_leaf(self, name: &str) -> DebugFunction;
    fn with_name(self, name: &str) -> DebugFunction;
}

impl FunctionsExt for Vec<DebugNamedFunction> {
    fn expect_leaf(mut self, name: &str) -> DebugFunction {
        let f = self.remove(0);
        assert_eq!(f.name, name);
        f.inner
    }

    fn with_name(self, name: &str) -> DebugFunction {
        self.into_iter()
            .find(|function| function.name == name)
            .map(|function| function.inner)
            .unwrap()
    }
}

pub trait DebugFunctionExt {
    fn active_expression(self) -> DebugMember;
    fn only_branch(self) -> DebugBranch;
}

impl DebugFunctionExt for DebugFunction {
    fn active_expression(self) -> DebugMember {
        self.branches
            .into_iter()
            .find_map(|branch| {
                branch
                    .body
                    .into_iter()
                    .find(|expression| expression.data.state.is_active())
            })
            .expect("Expected to find an active expression")
    }

    fn only_branch(mut self) -> DebugBranch {
        let branch = self.branches.remove(0);

        assert!(
            self.branches.is_empty(),
            "Expected one branch, but there are multiple."
        );

        branch
    }
}

pub trait DebugBranchExt {
    fn expression(&self, i: usize) -> DebugMember;
}

impl DebugBranchExt for DebugBranch {
    fn expression(&self, i: usize) -> DebugMember {
        let Some(expression) = self.body.get(i) else {
            panic!("{i}-th expression in `{:?}` not available", self.body);
        };

        expression.clone()
    }
}

pub trait DebugExpressionExt {
    fn expect_call_to_function(
        self,
        called_fn: &str,
        syntax_tree: &SyntaxTree,
        function_calls: &FunctionCalls,
    ) -> Self;
    fn expect_call_to_host_function(
        self,
        called_host_fn: &str,
        function_calls: &FunctionCalls,
    ) -> Self;
    fn expect_call_to_intrinsic(
        self,
        called_intrinsic: &str,
        function_calls: &FunctionCalls,
    ) -> Self;
    fn expect_function(self) -> DebugFunction;
}

impl DebugExpressionExt for DebugMember {
    fn expect_call_to_function(
        self,
        called_name: &str,
        syntax_tree: &SyntaxTree,
        function_calls: &FunctionCalls,
    ) -> Self {
        let Some(called_fn) = function_calls
            .is_call_to_user_defined_function(&self.data.location.clone())
        else {
            panic!("Expected call to function.");
        };
        let Some(user_fn) = syntax_tree.function_by_name(called_name) else {
            panic!("Function `{called_name}` does not exist.");
        };
        assert_eq!(*called_fn, user_fn.location());

        self
    }

    fn expect_call_to_host_function(
        self,
        called_host_fn: &str,
        function_calls: &FunctionCalls,
    ) -> Self {
        let Some(host_fn) =
            function_calls.is_call_to_host_function(&self.data.location)
        else {
            panic!("Expected call to host function.");
        };
        assert_eq!(called_host_fn, host_fn.name);

        self
    }

    fn expect_call_to_intrinsic(
        self,
        called_intrinsic: &str,
        function_calls: &FunctionCalls,
    ) -> Self {
        let Some(intrinsic_fn) =
            function_calls.is_call_to_intrinsic_function(&self.data.location)
        else {
            panic!("Expected call to intrinsic function.");
        };
        assert_eq!(called_intrinsic, intrinsic_fn.name());

        self
    }

    fn expect_function(self) -> DebugFunction {
        let DebugMemberKind::Function { function } = self.kind else {
            panic!("Expected function");
        };

        function
    }
}
