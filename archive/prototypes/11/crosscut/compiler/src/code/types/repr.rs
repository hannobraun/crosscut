use std::{collections::BTreeMap, fmt};

use crate::code::{
    syntax::{FunctionLocation, MemberLocation, ParameterLocation, SyntaxTree},
    Bindings, Dependencies, Identifiers,
};

use super::{
    infer::{infer, CompilerContext, InferenceOutput},
    resolve::resolve_type_annotations,
};

/// # The types that are explicitly specified in the code
///
/// ## Implementation Note
///
/// The long-term goal for Crosscut is to be fully inferred, with no explicit
/// types annotations being necessary at all. Type annotations are still used
/// for testing though, so it's likely that this code will stay, but the
/// compiler pipeline will forbid type annotations in regular code.
#[derive(Debug)]
pub struct TypeAnnotations {
    bindings: BTreeMap<ParameterLocation, Type>,
    expressions: BTreeMap<MemberLocation, Signature>,
}

impl TypeAnnotations {
    /// # Create an empty instance with no type annotations
    pub fn none() -> Self {
        Self {
            bindings: BTreeMap::default(),
            expressions: BTreeMap::default(),
        }
    }

    /// # Resolve all explicit type annotations
    pub fn resolve(syntax_tree: &SyntaxTree) -> Self {
        let (bindings, expressions) = resolve_type_annotations(syntax_tree);

        Self {
            bindings,
            expressions,
        }
    }

    /// # Access the annotation of the binding at the given location, if any
    pub fn of_binding(&self, location: &ParameterLocation) -> Option<&Type> {
        self.bindings.get(location)
    }

    /// # Access the annotation of the expression at the given location, if any
    pub fn of_expression(
        &self,
        location: &MemberLocation,
    ) -> Option<&Signature> {
        self.expressions.get(location)
    }

    /// # Iterate over the type annotations of all bindings
    pub fn of_all_bindings(
        &self,
    ) -> impl Iterator<Item = (&ParameterLocation, &Type)> {
        self.bindings.iter()
    }

    /// # Iterate over the type annotations of all expressions
    pub fn of_all_expressions(
        &self,
    ) -> impl Iterator<Item = (&MemberLocation, &Signature)> {
        self.expressions.iter()
    }
}

/// # The resolved types
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Types {
    functions: BTreeMap<FunctionLocation, Signature>,
    parameters: BTreeMap<ParameterLocation, Type>,
    expressions: BTreeMap<MemberLocation, Signature>,
    stacks: Stacks,
}

impl Types {
    /// # Infer types
    ///
    /// ## Implementation Note
    ///
    /// This current also uses the explicitly specified types where it can't
    /// infer yet, but the plan is to remove any explicit annotations in favor
    /// of full type inference.
    pub fn infer(
        syntax_tree: &SyntaxTree,
        bindings: &Bindings,
        identifiers: &Identifiers,
        dependencies: &Dependencies,
        annotations: TypeAnnotations,
    ) -> Self {
        let InferenceOutput {
            functions,
            expressions,
            parameters,
            stacks,
        } = infer(CompilerContext {
            syntax_tree,
            bindings,
            identifiers,
            dependencies,
            annotations: &annotations,
        });

        Self {
            functions,
            parameters,
            expressions,
            stacks,
        }
    }

    /// # Access the signature of the function at the given location, if any
    pub fn signature_of_function(
        &self,
        location: &FunctionLocation,
    ) -> Option<&Signature> {
        self.functions.get(location)
    }

    /// # Access the type of the parameter at the given location, if any
    pub fn type_of_parameter(
        &self,
        location: &ParameterLocation,
    ) -> Option<&Type> {
        self.parameters.get(location)
    }

    /// # Access the signature of the expression at the given location, if any
    pub fn signature_of_expression(
        &self,
        location: &MemberLocation,
    ) -> Option<&Signature> {
        self.expressions.get(location)
    }

    /// # Access the stack, as of the expression at the given location, if any
    ///
    /// This reflects what types are on the stack _before_ the expression is
    /// evaluated.
    ///
    /// The stack returned here is always local to the function that the
    /// expression is in. Tracking the global stack at compile-time is not
    /// possible, and only the local contents of the stack are relevant to what
    /// the expression can do anyway.
    pub fn stack_at(&self, location: &MemberLocation) -> Option<&[Type]> {
        self.stacks.get(location).map(|stack| &**stack)
    }
}

pub type Stacks = BTreeMap<MemberLocation, Stack>;
pub type Stack = Vec<Type>;

/// # A type signature that applies to a function or expression
///
/// This struct is generic over the type of the type in the signature. Usually
/// it's going to be [`Type`], but there are various specialized situation where
/// more specialized signatures are needed, and this type parameter enables that
/// without duplication.
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub struct Signature<T = Type> {
    /// # The inputs that the function or expression consumes
    pub inputs: Vec<T>,

    /// # The outputs that the function or expression produces
    pub outputs: Vec<T>,
}

impl<I, O> From<(I, O)> for Signature
where
    I: IntoIterator<Item = Type>,
    O: IntoIterator<Item = Type>,
{
    fn from((inputs, outputs): (I, O)) -> Self {
        Self {
            inputs: inputs.into_iter().collect(),
            outputs: outputs.into_iter().collect(),
        }
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut inputs = self.inputs.iter().peekable();
        while let Some(input) = inputs.next() {
            write!(f, "{input}")?;

            if inputs.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        write!(f, " ->")?;
        if !self.outputs.is_empty() {
            write!(f, " ")?;
        }

        let mut outputs = self.outputs.iter().peekable();
        while let Some(output) = outputs.next() {
            write!(f, "{output}")?;

            if outputs.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

/// # The type of a value
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    udigest::Digestable,
)]
pub enum Type {
    /// # A function
    Function {
        /// # The function's signature
        signature: Signature,
    },

    /// # A number
    ///
    /// ## Implementation Note
    ///
    /// Since the language is still mostly untyped, this actually covers many
    /// different types that are all represented as a 32-bit number.
    ///
    /// I expect that this will get split into multiple, more specific numeric
    /// types at some point.
    Number,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function { signature } => {
                write!(f, "fn {signature} end")?;
            }
            Self::Number => {
                write!(f, "Number")?;
            }
        }

        Ok(())
    }
}
