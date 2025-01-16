mod compile;

#[cfg(test)]
pub mod tests;

pub use self::compile::compile_and_replace;
