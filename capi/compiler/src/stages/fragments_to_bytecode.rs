use std::collections::BTreeSet;

use capi_process::{Bytecode, Function, Instruction, Location};

use crate::{
    repr::fragments::{
        Fragment, FragmentId, FragmentPayload, Fragments, FunctionFragments,
    },
    source_map::SourceMap,
};

pub fn fragments_to_bytecode(fragments: Fragments) -> (Bytecode, SourceMap) {
    let mut bytecode = Bytecode::default();
    let mut source_map = SourceMap::default();

    let mut compiler = Compiler {
        bytecode: &mut bytecode,
        source_map: &mut source_map,
    };

    for function in fragments.by_function {
        compiler.compile_function(
            function.name,
            function.args,
            function.fragments,
        );
    }

    (bytecode, source_map)
}

struct Compiler<'r> {
    bytecode: &'r mut Bytecode,
    source_map: &'r mut SourceMap,
}

impl Compiler<'_> {
    fn compile_function(
        &mut self,
        name: String,
        args: Vec<String>,
        fragments: FunctionFragments,
    ) {
        let mut bindings = args.iter().cloned().collect();
        let mut output = Function::new(name.clone(), args);

        for fragment in fragments {
            self.compile_fragment(fragment, &mut bindings, &mut output);
        }

        self.bytecode.functions.insert(name, output);
    }

    fn compile_fragment(
        &mut self,
        fragment: Fragment,
        bindings: &mut BTreeSet<String>,
        output: &mut Function,
    ) {
        let fragment_id = fragment.id();

        match fragment.payload {
            FragmentPayload::BindingDefinitions { names } => {
                for name in names.iter().cloned().rev() {
                    // Inserting bindings unconditionally like that does mean
                    // that bindings can overwrite previously defined bindings.
                    // This is undesirable, but it'll do for now.
                    bindings.insert(name);
                }

                self.generate(
                    Instruction::BindingsDefine { names },
                    fragment_id,
                    output,
                );
            }
            FragmentPayload::Comment { .. } => {}
            FragmentPayload::FunctionCall { name } => {
                self.generate(
                    Instruction::CallFunction { name },
                    fragment_id,
                    output,
                );
            }
            FragmentPayload::Value(value) => {
                self.generate(Instruction::Push { value }, fragment_id, output);
            }
            FragmentPayload::Word { name } => {
                let instruction = word_to_instruction(name, bindings);
                self.generate(instruction, fragment_id, output);
            }
        };
    }

    fn generate(
        &mut self,
        instruction: Instruction,
        fragment_id: FragmentId,
        output: &mut Function,
    ) {
        let index = output.instructions.push(instruction);

        let runtime_location = Location {
            function: output.name.clone(),
            index,
        };
        self.source_map
            .define_mapping(runtime_location.clone(), fragment_id);
    }
}

fn word_to_instruction(
    word: String,
    bindings: &BTreeSet<String>,
) -> Instruction {
    // Here we check for special built-in functions that are implemented
    // differently, without making sure anywhere, that its name doesn't conflict
    // with any user-defined functions.
    //
    // I think it's fine for now. This seems like a temporary hack anyway, while
    // the language is not powerful enough to support an actual `if`.
    if word == "return_if_non_zero" {
        return Instruction::ReturnIfNonZero;
    }
    if word == "return_if_zero" {
        return Instruction::ReturnIfZero;
    }

    // The code here would allow bindings to shadow both user-defined and
    // builtin functions. This seems undesirable, without further handling to
    // prevent mistakes.
    //
    // It's better to catch this when defining bindings, though.
    if bindings.contains(&word) {
        return Instruction::BindingEvaluate { name: word };
    }

    // This doesn't check whether the built-in function exists, and given how
    // built-in functions are currently defined, it's not practical to
    // implement.
    Instruction::CallBuiltin { name: word }
}
