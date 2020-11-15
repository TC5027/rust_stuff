/// Work stealing threadpool with 2 threads, it is just a learning exercise for myself
/// join(a,b) : met b dans la deque du thread ou se passe join(a,b) 
/// et execute a
use std::sync::{Mutex,Arc};
use std::cell::RefCell;
use std::thread::{JoinHandle,spawn};


thread_local!(static FOO : RefCell<Arc<Mutex<Storage<Box<dyn FnOnce() + Send>>>>> = RefCell::new(Arc::new(Mutex::new(Storage::new()))));

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

#[allow(dead_code)]
struct Threadpoolworkstealing {
    storages : (Arc<Mutex<Storage<Box<dyn FnOnce() + 'static + Send>>>>, Arc<Mutex<Storage<Box<dyn FnOnce() + 'static + Send>>>>),
    handlers : (JoinHandle<()>, JoinHandle<()>),
}

impl Threadpoolworkstealing {
    pub fn new() -> Threadpoolworkstealing {
        let (storage1, storage2) = (Storage::new(), Storage::new());
        let (storage1, storage2) = (Arc::new(Mutex::new(storage1)), Arc::new(Mutex::new(storage2)));
        
        let storage1_for_get = storage1.clone();
        let storage1_for_steal = storage1.clone();

        let storage2_for_get = storage2.clone();
        let storage2_for_steal = storage2.clone();
        
        Threadpoolworkstealing {
            storages : (storage1, storage2),
            handlers : (
                spawn(move || {
                    FOO.with(|f| {
                        *f.borrow_mut() = storage1_for_get.clone();
                    });
                    loop {
                        if storage1_for_get.lock().unwrap().is_empty() {
                            let steal = storage2_for_steal.lock().unwrap().steal();
                            // putting the lock on the match makes it last too much
                            match steal {
                                Some(f) => {storage1_for_get.lock().unwrap().add(f);},
                                None => {}
                            }
                        } else {
                            let get = storage1_for_get.lock().unwrap().get();
                            match get {
                                Some(f) => {println!("1");f();}
                                None => {}
                            }
                        }
                    }
                }),
                spawn(move || {
                    FOO.with(|f| {
                        *f.borrow_mut() = storage2_for_get.clone();
                    });
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
                                Some(f) => {println!("2");f();}
                                None => {}
                            }
                        }
                    }
                })
            )
        }
    }

    pub fn add<F>(&self, job : F) 
        where F : FnOnce() + 'static + Send
    {
        self.storages.0.lock().unwrap().add(Box::new(job));
    }

}

fn join<F1,F2>(job1 : F1, job2 : F2) -> Box<dyn 'static + Send + FnOnce() -> ()>
    where F1 : FnOnce() + 'static + Send,
          F2 : FnOnce() + 'static + Send,
{
    Box::new(|| {
        FOO.with(|f| {
            f.borrow().lock().unwrap().add(Box::new(
                job2
            ));
        });
        job1();
    })
}

fn main() {
    let threadpool = Threadpoolworkstealing::new();
    threadpool.add(join(
        ||{std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");},
        ||{std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");}
    ));

    //threadpool.add(|| {std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");});
    //threadpool.add(|| {std::thread::sleep(std::time::Duration::from_secs(2)); print!("a");});
    std::thread::sleep(std::time::Duration::from_secs(3));
}
