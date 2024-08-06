use std::collections::{BTreeMap, BTreeSet, VecDeque};

use capi_process::{Bytecode, Instruction, InstructionAddress, Instructions};

use crate::{
    fragments::{
        Fragment, FragmentExpression, FragmentId, FragmentMap, FragmentPayload,
        Fragments, Function,
    },
    placeholders::{CallToUserDefinedFunction, Placeholders},
    source_map::SourceMap,
    syntax::Pattern,
};

pub fn generate_bytecode(fragments: Fragments) -> (Bytecode, SourceMap) {
    let mut output = Output {
        instructions: Instructions::default(),
        placeholders: Placeholders::default(),
        source_map: SourceMap::default(),
    };

    // Create placeholder for call to `main` function, and the last return that
    // ends the process, if executed.
    let main = output.instructions.push(Instruction::Panic);
    output.instructions.push(Instruction::Return);

    let mut compiler = Compiler {
        queue: VecDeque::new(),
        output,
        function_arguments_by_address: BTreeMap::new(),
        function_addresses_by_name: BTreeMap::new(),
        fragments: &fragments.inner,
    };

    compiler.compile_context(fragments.root);
    compiler.compile();

    if let Some(address) = compiler.function_addresses_by_name.get("main") {
        // If we have an entry function, replace that panic instruction we added
        // as a placeholder.
        //
        // Right now, this will just result in an non-descriptive panic, if no
        // entry function was provided. Eventually, the panic instruction might
        // grow a "reason" parameter which will provide more clarity in such a
        // case.
        //
        // In addition, this is something that should be detected during pre-
        // compilation, and result in a nice error message in the debugger.
        compiler.output.instructions.replace(
            main,
            Instruction::CallFunction {
                address: *address,
                is_tail_call: true,
            },
        );
    }

    for call in compiler.output.placeholders.inner {
        let Some(address) = compiler.function_addresses_by_name.get(&call.name)
        else {
            unreachable!(
                "Expecting function `{}` to exist. If it didn't, the previous \
                compilation step would not have generated the fragment that \
                caused us to assume that it does.",
                call.name,
            );
        };

        compiler.output.instructions.replace(
            call.address,
            Instruction::CallFunction {
                address: *address,
                is_tail_call: call.is_tail_call,
            },
        );
    }

    let bytecode = Bytecode {
        instructions: compiler.output.instructions,
        function_arguments: compiler.function_arguments_by_address,
    };

    (bytecode, compiler.output.source_map)
}

struct Compiler<'r> {
    queue: VecDeque<CompileUnit>,
    output: Output,
    function_arguments_by_address: BTreeMap<InstructionAddress, Vec<String>>,
    function_addresses_by_name: BTreeMap<String, InstructionAddress>,
    fragments: &'r FragmentMap,
}

impl Compiler<'_> {
    fn compile(&mut self) {
        while let Some(unit) = self.queue.pop_front() {
            match unit {
                CompileUnit::Block {
                    start,
                    environment,
                    address,
                } => {
                    let start = self.compile_context(start);

                    self.output.instructions.replace(
                        address,
                        Instruction::MakeClosure {
                            address: start,
                            environment,
                        },
                    );
                }
                CompileUnit::Function(function) => {
                    self.compile_function(function);
                }
            }
        }
    }

    fn compile_function(&mut self, function: Function) {
        let address = self.compile_context(function.start);
        let arguments = function
            .arguments
            .into_iter()
            .filter_map(|pattern| match pattern {
                Pattern::Identifier { name } => Some(name),
                Pattern::Literal { .. } => {
                    // The parameter list of a function is used to provide the
                    // arguments to the function at runtime. But literal
                    // patterns aren't relevant to the function itself. They are
                    // only used to select which function to call in the first
                    // place.
                    None
                }
            })
            .collect();

        self.function_arguments_by_address
            .insert(address, arguments);
        self.function_addresses_by_name
            .insert(function.name, address);
    }

    fn compile_context(&mut self, start: FragmentId) -> InstructionAddress {
        let mut first_instruction = None;

        for fragment in self.fragments.iter_from(start) {
            let addr = self.compile_fragment(fragment);
            first_instruction = first_instruction.or(addr);
        }

        let Some(first_instruction) = first_instruction else {
            unreachable!(
                "Must have generated at least one instruction for the block: \
                the return instruction. If this has not happened, the \
                fragments have somehow been missing a terminator."
            );
        };

        first_instruction
    }

    fn compile_fragment(
        &mut self,
        fragment: &Fragment,
    ) -> Option<InstructionAddress> {
        let addr = match &fragment.payload {
            FragmentPayload::Expression { expression, .. } => {
                match expression {
                    FragmentExpression::BindingDefinitions { names } => {
                        generate_instruction(
                            Instruction::BindingsDefine {
                                names: names.clone(),
                            },
                            fragment.id(),
                            &mut self.output,
                        )
                    }
                    FragmentExpression::Block { start, environment } => {
                        // We are currently compiling a function or block
                        // (otherwise we wouldn't be encountering any
                        // expression), and the instructions for that will be
                        // executed linearly.
                        //
                        // Which means we can't just start compiling this block
                        // right now. Its instructions would go into the middle
                        // of those other instructions and mess everything up.
                        //
                        // What _should_ happen instead, is that the block is
                        // turned into a closure, that can be passed around as a
                        // value and called whenever so desired.
                        //
                        // So for now, let's just generate this instruction as
                        // a placeholder, to be replaced with another
                        // instruction that creates that closure, once we have
                        // everything in place to make that happen.
                        let address = generate_instruction(
                            Instruction::Panic,
                            fragment.id(),
                            &mut self.output,
                        );

                        // And to make it happen later, we need to put what we
                        // already have into a queue. Once whatever's currently
                        // being compiled is out of the way, we can process
                        // that.
                        self.queue.push_front(CompileUnit::Block {
                            start: *start,
                            environment: environment.clone(),
                            address,
                        });

                        address
                    }
                    FragmentExpression::Comment { .. } => {
                        return None;
                    }
                    FragmentExpression::ResolvedBinding { name } => {
                        generate_instruction(
                            Instruction::BindingEvaluate { name: name.clone() },
                            fragment.id(),
                            &mut self.output,
                        )
                    }
                    FragmentExpression::ResolvedBuiltinFunction { name } => {
                        // Here we check for special built-in functions that are
                        // implemented differently, without making sure
                        // anywhere, that their name doesn't conflict with any
                        // user-defined functions.
                        //
                        // I think it's fine for now. This seems like a
                        // temporary hack anyway, while the language is not
                        // powerful enough to support real conditionals.
                        let instruction = if name == "return_if_non_zero" {
                            Instruction::ReturnIfNonZero
                        } else if name == "return_if_zero" {
                            Instruction::ReturnIfZero
                        } else {
                            Instruction::CallBuiltin { name: name.clone() }
                        };

                        generate_instruction(
                            instruction,
                            fragment.id(),
                            &mut self.output,
                        )
                    }
                    FragmentExpression::ResolvedHostFunction { name } => {
                        generate_instruction(
                            Instruction::CallBuiltin { name: name.clone() },
                            fragment.id(),
                            &mut self.output,
                        )
                    }
                    FragmentExpression::ResolvedUserFunction {
                        name,
                        is_tail_call,
                    } => {
                        // We know that this expression refers to a user-defined
                        // function, but we might not have compiled that
                        // function yet.
                        //
                        // For now, just generate a placeholder that we can
                        // replace with the call later.
                        let address = generate_instruction(
                            Instruction::Panic,
                            fragment.id(),
                            &mut self.output,
                        );

                        // We can't leave it at that, however. We need to make
                        // sure this placeholder actually gets replace later,
                        // and we're doing that by adding it to this list.
                        self.output.placeholders.inner.push(
                            CallToUserDefinedFunction {
                                name: name.clone(),
                                address,
                                is_tail_call: *is_tail_call,
                            },
                        );

                        address
                    }
                    FragmentExpression::UnresolvedIdentifier { name: _ } => {
                        generate_instruction(
                            Instruction::Panic,
                            fragment.id(),
                            &mut self.output,
                        )
                    }
                    FragmentExpression::Value(value) => generate_instruction(
                        Instruction::Push { value: *value },
                        fragment.id(),
                        &mut self.output,
                    ),
                }
            }
            FragmentPayload::Function(function) => {
                self.queue
                    .push_back(CompileUnit::Function(function.clone()));
                return None;
            }
            FragmentPayload::Terminator => generate_instruction(
                Instruction::Return,
                fragment.id(),
                &mut self.output,
            ),
        };

        Some(addr)
    }
}

fn generate_instruction(
    instruction: Instruction,
    fragment_id: FragmentId,
    output: &mut Output,
) -> InstructionAddress {
    let addr = output.instructions.push(instruction);
    output.source_map.define_mapping(addr, fragment_id);
    addr
}

struct Output {
    instructions: Instructions,
    placeholders: Placeholders,
    source_map: SourceMap,
}

enum CompileUnit {
    Block {
        start: FragmentId,
        environment: BTreeSet<String>,
        address: InstructionAddress,
    },
    Function(Function),
}
