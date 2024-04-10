use capi_runtime::{Evaluator, Functions};

pub struct Program {
    pub functions: Functions,
    pub evaluator: Evaluator,
    pub entry: usize,
}

impl Program {
    pub fn new() -> Self {
        let mut functions = Functions::new();

        functions.define("write_to_tile_buffer", |s| {
            s.w("last_tile_index")
                .w("first_tile_index")
                .w("set_all_tiles")
                .w("drop2");
        });
        functions.define("last_tile_index", |s| {
            s.w("mul").w("first_tile_index").w("add");
        });
        functions.define("first_tile_index", |s| {
            s.v(256);
        });
        functions.define("set_all_tiles", |s| {
            s.w("check_tile_index")
                .w("return_if_zero")
                .w("set_tile")
                .w("set_all_tiles");
        });
        functions.define("check_tile_index", |s| {
            s.v(1).w("copy").v(1).w("copy").w("sub");
        });
        functions.define("set_tile", |s| {
            s.v(1).w("store").w("inc_tile_index");
        });
        functions.define("inc_tile_index", |s| {
            s.v(1).w("add");
        });

        let code = functions.clone().compile();
        let entry = code.symbols.resolve("write_to_tile_buffer");

        let evaluator = Evaluator::new(code);

        Self {
            functions,
            evaluator,
            entry,
        }
    }

    pub fn run(
        &mut self,
        frame_width: usize,
        frame_height: usize,
        frame: &mut [u8],
    ) {
        self.evaluator.data_stack.push(frame_width);
        self.evaluator.data_stack.push(frame_height);
        self.evaluator.evaluate(self.entry, frame);

        assert_eq!(self.evaluator.data_stack.num_values(), 0);
    }
}
