use std::{collections::VecDeque, sync::Mutex};

static STATE: Mutex<Option<VecDeque<u8>>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn on_key(code: u8) {
    let mut state = STATE.lock().unwrap();
    let state = state.get_or_insert_with(Default::default);
    state.push_back(code);

    log::info!("{state:?}");
}
