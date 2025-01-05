pub fn create<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam_channel::unbounded()
}

pub type Sender<T> = crossbeam_channel::Sender<T>;
pub type Receiver<T> = crossbeam_channel::Receiver<T>;
