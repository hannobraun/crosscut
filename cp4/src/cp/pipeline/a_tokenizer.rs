use std::iter;

pub struct Tokenizer {
    buf: String,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn tokenize<'r>(
        &'r mut self,
        code: impl Iterator<Item = char> + 'r,
    ) -> impl Iterator<Item = String> + 'r {
        let mut code = code;
        let mut tokenizer = Tokenizer::new();

        iter::from_fn(move || loop {
            let ch = match code.next() {
                Some(ch) => ch,
                None => {
                    if tokenizer.buf.is_empty() {
                        return None;
                    }

                    let token = tokenizer.buf.clone();
                    tokenizer.buf.clear();
                    return Some(token);
                }
            };

            if ch.is_whitespace() {
                let token = tokenizer.buf.clone();
                tokenizer.buf.clear();
                return Some(token);
            }

            tokenizer.buf.push(ch);
        })
    }
}
