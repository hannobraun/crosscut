use std::fmt;

use crate::code::{Signature, Type};

macro_rules! intrinsics {
    (
        $(
            #[$attrs:meta]
            $name:expr,
            $variant:ident,
            $signature:expr;
        )*
    ) => {
        /// # Special functions that are known to the compiler
        ///
        /// When encountering a call to an intrinsic, the compiler will directly
        /// translate that into the appropriate instructions.
        #[derive(
            Clone,
            Copy,
            Debug,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            serde::Deserialize,
            serde::Serialize,
            udigest::Digestable,
        )]
        pub enum IntrinsicFunction {
            $($variant,)*
        }

        impl IntrinsicFunction {
            pub fn from_name(name: &str) -> Option<IntrinsicFunction> {
                let intrinsic = match name {
                    $($name => Self::$variant,)*

                    _ => {
                        return None;
                    }
                };

                Some(intrinsic)
            }

            /// # Access the name of this intrinsic function
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => {
                            $name
                        }
                    )*
                }
            }

            /// # Access the type signature of this intrinsic function
            ///
            /// Not all intrinsic functions can provide a type signature, as the
            /// type system is not advanced enough to express them. In this
            /// case, this method returns `None`, and the caller needs to
            /// implement special handling.
            pub fn signature(&self) -> Option<Signature> {
                match self {
                    $(
                        Self::$variant => {
                            $signature.map(Signature::from)
                        }
                    )*
                }
            }
        }

        impl fmt::Display for IntrinsicFunction {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $name),)*
                }
            }
        }
    };
}

use Type::*;

intrinsics! {
    /// # Add two signed 8-bit integers, triggering an error on overflow
    "add_s8", AddS8, Some(([Number, Number], [Number]));

    /// # Add two signed 32-bit integers, triggering an error on overflow
    "add_s32", AddS32, Some(([Number, Number], [Number]));

    /// # Add two unsigned 8-bit integers, triggering an error on overflow
    "add_u8", AddU8, Some(([Number, Number], [Number]));

    /// # Add two unsigned 8-bit integers, wrapping on overflow
    "add_u8_wrap", AddU8Wrap, Some(([Number, Number], [Number]));

    /// # Logical and
    "and", And, Some(([Number, Number], [Number]));

    /// # Trigger a breakpoint
    "brk", Brk, Some(([], []));

    /// # Copy a value
    "copy", Copy, Option::<([Type; 0], [Type; 0])>::None;

    /// # Divide two signed 32-bit integers
    "div_s32", DivS32, Some(([Number, Number], [Number]));

    /// # Divide two unsigned 8-bit integers
    "div_u8", DivU8, Some(([Number, Number], [Number]));

    /// # Drop a value
    "drop", Drop, Option::<([Type; 0], [Type; 0])>::None;

    /// # Compare two values for equality
    "eq", Eq, Some(([Number, Number], [Number]));

    /// # Evaluate an anonymous function
    "eval", Eval, Option::<([Type; 0], [Type; 0])>::None;

    /// # Determine if the first of two signed 8-bit numbers is greater
    "greater_s8", GreaterS8, Some(([Number, Number], [Number]));

    /// # Determine if the first of two signed 32-bit numbers is greater
    "greater_s32", GreaterS32, Some(([Number, Number], [Number]));

    /// # Determine if the first of two unsigned 8-bit numbers is greater
    "greater_u8", GreaterU8, Some(([Number, Number], [Number]));

    /// # Multiply two signed 32-bit numbers, triggering an error on overflow
    "mul_s32", MulS32, Some(([Number, Number], [Number]));

    /// # Multiply two unsigned 8-bit numbers, wrapping on overflow
    "mul_u8_wrap", MulU8Wrap, Some(([Number, Number], [Number]));

    /// # Negate a signed 32-bit number
    "neg_s32", NegS32, Some(([Number], [Number]));

    /// No operation
    "nop", Nop, Some(([], []));

    /// # Logical not
    "not", Not, Some(([Number], [Number]));

    /// # Compute the remainder of the division of two signed 32-bit numbers
    "remainder_s32", RemainderS32, Some(([Number, Number], [Number]));

    /// # Convert a signed 32-bit number to a signed 8-bit number
    "s32_to_s8", S32ToS8, Some(([Number], [Number]));

    /// # Subtract two signed 32-bit numbers, triggering an error on overflow
    "sub_s32", SubS32, Some(([Number, Number], [Number]));

    /// # Subtract two unsigned 8-bit numbers, triggering an error on overflow
    "sub_u8", SubU8, Some(([Number, Number], [Number]));

    /// # Subtract two unsigned 8-bit numbers, wrapping on overflow
    "sub_u8_wrap", SubU8Wrap, Some(([Number, Number], [Number]));
}
