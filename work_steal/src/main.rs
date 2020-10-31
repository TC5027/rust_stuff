/// Work stealing threadpool with 2 threads, it is just a learning exercise for myself

use std::sync::{Mutex,Arc};
use std::thread::{JoinHandle,spawn};

// Ideally use a Deque instead
struct Storage<T> {
    vec : Vec<T>,
}

impl<T> Storage<T> {
    pub fn new() -> Self {
        Storage{
            vec : vec![]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn steal(&mut self) -> Option<T> {
        match self.vec.len() {
            0 => None,
            _ => Some(self.vec.remove(0))
        }
    }

    pub fn get(&mut self) -> Option<T> {
        self.vec.pop()
    }

    pub fn add(&mut self, t:T) {
        self.vec.push(t);
    }
}

struct Threadpoolworkstealing {
    entry_storage : Arc<Mutex<Storage<Box<dyn FnOnce() + 'static + Send>>>>,
    handler1 : JoinHandle<()>,
    handler2 : JoinHandle<()>
}

impl Threadpoolworkstealing {
    pub fn new() -> Threadpoolworkstealing {
        let (storage1, storage2) = (Storage::new(), Storage::new());
        let entry_storage = Arc::new(Mutex::new(storage1));
        
        let storage1_for_get = entry_storage.clone();
        let storage1_for_steal = entry_storage.clone();

        let storage2_for_get = Arc::new(Mutex::new(storage2));
        let storage2_for_steal = storage2_for_get.clone();

        Threadpoolworkstealing {
            entry_storage,
            handler1 : spawn(move || {
                loop {
                    if storage1_for_get.lock().unwrap().is_empty() {
                        let steal = storage2_for_steal.lock().unwrap().steal();
                        // putting the lock on the match seems to make it last too much, I'm not sure
                        match steal {
                            Some(f) => {storage1_for_get.lock().unwrap().add(f);},
                            None => {}
                        }
                    } else {
                        let get = storage1_for_get.lock().unwrap().get();
                        match get {
                            Some(f) => {f();}
                            None => {}
                        }
                    }
                }
            }),
            handler2 : spawn(move || {
                loop {
                    if storage2_for_get.lock().unwrap().is_empty() {
                        let steal = storage1_for_steal.lock().unwrap().steal();
                        match steal {
                            Some(f) => {storage2_for_get.lock().unwrap().add(f);},
                            None => {}
                        }
                    } else {
                        let get = storage2_for_get.lock().unwrap().get();
                        match  get {
                            Some(f) => {f();}
                            None => {}
                        }
                    }
                }
            })
        }
    }

    pub fn add<F>(&self, job : F) 
        where F : FnOnce() + 'static + Send
    {
        self.entry_storage.lock().unwrap().add(Box::new(job));
    }
}
fn main() {
    let threadpool = Threadpoolworkstealing::new();
    threadpool.add(|| {std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");});
    threadpool.add(|| {std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");});
    std::thread::sleep(std::time::Duration::from_secs(3));
}
