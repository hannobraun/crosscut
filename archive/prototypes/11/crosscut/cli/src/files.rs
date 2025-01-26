use std::path::Path;

pub const FILES: Files = Files::new();

macro_rules! files {
    ($($field:ident, $name:expr;)*) => {
        #[derive(Debug)]
        pub struct Files {
            $(
                $field: &'static [u8],
            )*
        }

        impl Files {
            pub const fn new() -> Self {
                Self {
                    $(
                        $field: include_bytes!(concat!(
                            env!("FILES"),
                            "/",
                            $name,
                        )),
                    )*
                }
            }

            pub fn list_invalid(&self) -> Vec<&'static str> {
                let mut invalid_files = Vec::new();

                let files = [
                    $(
                        (self.$field, $name),
                    )*
                ];

                for (file, name) in files {
                    if file.is_empty() {
                        invalid_files.push(name);
                    }
                }

                invalid_files
            }

            pub fn get(&self, path: impl AsRef<Path>) -> Option<&[u8]> {
                match path.as_ref().to_str() {
                    $(
                        Some($name) => Some(self.$field),
                    )*
                    _ => None,
                }
            }
        }
    };
}

files!(
    a, "crosscut-debugger_bg.wasm";
    b, "crosscut-debugger.js";
    c, "crosscut_host.wasm";
    d, "index.html";
    e, "index-debugger.html";
    f, "tailwind.js";
);
