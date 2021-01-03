use std::thread;
use rand::Rng;
use std::collections::VecDeque;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

type Shorter = Arc<Mutex<VecDeque<Task>>>;

thread_local!(static LOCAL_DEQUE : RefCell<Shorter>
    = RefCell::new(Arc::new(Mutex::new(VecDeque::new()))));

pub struct Task {
    index : u8,
    boxed_t : Box<dyn FnOnce() + Send + 'static>,
    timer : Arc<Mutex<Duration>>,
    start : Instant,
    duration : Arc<Mutex<Duration>>
}

impl Task {
    pub fn new(index : u8, boxed_t : Box<dyn FnOnce() + Send + 'static>, timer : Arc<Mutex<Duration>>, start : Instant, duration : Arc<Mutex<Duration>>) -> Self {
        Task {
            index,
            boxed_t,
            timer,
            start,
            duration
        }
    }

    pub fn execute(self) {
        let start = Instant::now();
        let f = self.boxed_t;
        println!("{:?}",self.index);
        f();
        let duration = start.elapsed();
        // we add to timer the ACCUMULATED time of execution, timer helps us
        // characterize LARGE tasks or not
        *self.timer.lock().unwrap() += duration;
        // we update duration as the time since the task has been declared and is so
        // waiting to be completed (possibly in less than the target latency)
        *self.duration.lock().unwrap() = self.start.elapsed();
    }

    pub fn is_stealable(&self, threshold : Duration) -> bool {
        *self.timer.lock().unwrap() < threshold
    }
}

fn get_index_of_thread(mine : usize, steal_index : usize) -> usize {
    if steal_index>=mine {steal_index+1} else {steal_index}
}

fn schedule(local_deque : Shorter, other_deques : Vec<Shorter>, global_queue : Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>>, active_tasks : Arc<AtomicU8>, declared_unstealable : Arc<Mutex<Vec<usize>>>, index_of_thread : usize) {
    let mut rng = rand::thread_rng();
    loop {
        if local_deque.lock().unwrap().is_empty() {
            // the thread's tasks are not unstealable anymore (if it was the case)
            declared_unstealable.lock().unwrap().retain(|&i| i!= index_of_thread);
            active_tasks.fetch_sub(1,Ordering::Relaxed);

            let mut steal_index = rng.gen_range(0..other_deques.len());
            // we loop while the victim thread contains unstealable task
            while declared_unstealable.lock().unwrap().contains(&get_index_of_thread(index_of_thread, steal_index)) {
                steal_index = rng.gen_range(0..other_deques.len());
            }

            let stealed_task = other_deques[steal_index].lock().unwrap().pop_back();
            match stealed_task {
                Some(f) => {
                    // this is where we declare the target latency
                    if f.is_stealable(Duration::from_secs(4)) {
                        f.execute();
                    } else {
                        // we set the victim thread's tasks as unstealable
                        declared_unstealable.lock().unwrap().push(get_index_of_thread(index_of_thread, steal_index));
                        // we add back the task
                        other_deques[steal_index].lock().unwrap().push_back(f);
                    }},
                None => {
                    if active_tasks.compare_and_swap(0,1,Ordering::Relaxed)==0 {
                        let from_global_task = global_queue.lock().unwrap().pop_front();
                        match from_global_task {
                            Some(f) => {f();},
                            None => {}
                        }
                    }
                }
            }
        } else {
            let task = local_deque.lock().unwrap().pop_front();
            match task {
                Some(f) => {f.execute();},
                None => {}
            }
        }
        
    }
}

#[allow(dead_code)]
pub struct Threadpool {
    handlers : Vec<thread::JoinHandle<()>>,
    global_queue : Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    timers : Vec<Arc<Mutex<Duration>>>,
    durations : Vec<Arc<Mutex<Duration>>>,
    nb_tasks : u8,
    active_tasks : Arc<AtomicU8>
}

impl Threadpool {
    pub fn new(number_thread : usize) -> Self {
        let global_queue = Arc::new(Mutex::new(VecDeque::new()));
        let active_tasks = Arc::new(AtomicU8::new(0));
        let declared_unstealable = Arc::new(Mutex::new(Vec::new()));
        let mut deques = (0..number_thread).map(|_| Arc::new(Mutex::new(VecDeque::new()))).collect::<Vec<_>>();
        let handlers = (0..number_thread).map(|i| {
            let global_queue = global_queue.clone();
            let active_tasks = active_tasks.clone();
            let declared_unstealable = declared_unstealable.clone();
            let mut other_deques = vec![];
            let mut local_deque = Arc::new(Mutex::new(VecDeque::new()));
            let mut storing = vec![];
            for k in 0..number_thread {
                let deque = deques.pop().unwrap();
                if (number_thread-k-1)==i {
                    local_deque=deque.clone();
                }
                else {
                    other_deques.push(deque.clone());
                }
                storing.push(deque);
            }
            storing.reverse();
            deques = storing;
            // we could be more efficient looking at parity of i.
            thread::spawn(move || {
                LOCAL_DEQUE.with( |deque| {
                    *deque.borrow_mut() = local_deque.clone();
                });
                schedule(local_deque, other_deques, global_queue, active_tasks,declared_unstealable,i);
            })
        }).collect();
        Threadpool{
            handlers,
            global_queue,
            timers : Vec::new(),
            durations : Vec::new(),
            nb_tasks : 0,
            active_tasks
        }
    }

    pub fn forall<'a, F, A, I,>(&mut self, iter : I, f : F) 
        where F : FnOnce(A) + Send + Clone + 'static,
              A : Send + 'static,
              I : Send + IntoIterator<Item = A> + 'static
    {
        let index = self.nb_tasks;
        let start = Instant::now();
        let new_timer = Arc::new(Mutex::new(start.elapsed()));
        let new_duration = Arc::new(Mutex::new(start.elapsed()));
        self.nb_tasks += 1;
        self.timers.push(new_timer.clone());
        self.durations.push(new_duration.clone());
        self.global_queue.lock().unwrap().push_back(Box::new(move || 
            LOCAL_DEQUE.with( |deque| {
                for t in iter.into_iter() {
                    let f = f.clone();
                    deque.borrow().lock().unwrap().push_back(Task::new(index, Box::new(|| f(t)), new_timer.clone(), start, new_duration.clone()));
                }
            })
        ));
    }
}

fn example_of_use() {
    let mut threadpool = Threadpool::new(3);
    threadpool.forall(0..6, |_| {std::thread::sleep(Duration::from_secs(2))});
    std::thread::sleep(Duration::from_millis(1));
    threadpool.forall(0..2, |_| {std::thread::sleep(Duration::from_secs(1))});
    std::thread::sleep(Duration::from_secs(10));
    println!("with a target latency of 4 secs, and a policy which serializes 
    if the operation took more than 4 secs (accumulated) we get :\n{:?}\nso 
    the operation 0 missed target latency but operation 1 succeed to hit it, 
    thanks to the serialization of operation 0, without hit both would
     have missed", threadpool.durations);
}

fn main() {
    // let mut threadpool = Threadpool::new(2);
    // let counter = Arc::new(AtomicU8::new(0));
    // let to_print = counter.clone();

    // let c = counter.clone();
    // threadpool.forall(0..4, move |_| {c.fetch_add(1, Ordering::Relaxed);});
    // let c = counter.clone();
    // threadpool.forall(0..4, move |_| {c.fetch_add(1, Ordering::Relaxed);});
    // threadpool.forall(0..4, move |_| {std::thread::sleep(Duration::from_secs(1))});
    // std::thread::sleep(Duration::from_secs(6));
    // println!("{:?}", to_print);
    // println!("{:?}", threadpool.timers);
    example_of_use();
}
