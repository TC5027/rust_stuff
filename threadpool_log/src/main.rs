use crossbeam_channel::{bounded, Receiver, Sender};
use rand::{prelude::ThreadRng, thread_rng, Rng};
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Result, Write};
use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    Arc, Mutex,
};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::{Duration, Instant};

mod events;
use events::*;

mod svg;
use svg::*;

type Shared<T> = Arc<Mutex<T>>;
type LocalDeque = Arc<Mutex<VecDeque<Task>>>;

thread_local! {
    static LOCAL_DEQUE: RefCell<LocalDeque> =
        RefCell::new(Arc::new(Mutex::new(VecDeque::new())));

    static LOCAL_EVENTLOG: RefCell<Shared<EventLog>> =
        RefCell::new(Arc::new(Mutex::new(EventLog::new())));
}

struct Task {
    inner_task: Box<dyn Fn() + Send + 'static>,
    request_declaration: Instant,
    counter_left_brother_tasks: Arc<AtomicUsize>,
    request_color: Color,
}

impl Task {
    fn new<T: Fn() + Send + 'static>(
        inner_task: T,
        request_declaration: Instant,
        counter_left_brother_tasks: Arc<AtomicUsize>,
        request_color: Color,
    ) -> Self {
        Self {
            inner_task: Box::new(inner_task),
            request_declaration,
            counter_left_brother_tasks,
            request_color,
        }
    }

    fn execute(
        self,
        tasks_counter: Arc<AtomicUsize>,
        eventlog: Shared<EventLog>,
        steal: Option<usize>,
    ) {
        let time = Instant::now();
        // if the Option is of variant Some, that means the execute
        // was previoused by a steal so we add a Steal to the eventlog
        if steal.is_some() {
            eventlog.lock().unwrap().push(Event {
                category: EventCategory::Steal(steal.unwrap()),
                time,
                color: self.request_color,
            })
        }
        // we add a StartProcessing to the eventlog
        eventlog.lock().unwrap().push(Event {
            category: EventCategory::StartProcessing,
            time,
            color: self.request_color,
        });
        // execution
        (self.inner_task)();
        let time = Instant::now();
        // we add an EndProcessing to the eventlog
        eventlog.lock().unwrap().push(Event {
            category: EventCategory::EndProcessing,
            time,
            color: self.request_color,
        });
        if self.counter_left_brother_tasks.load(Relaxed) != 0 {
            // we decrease by one the counter of tasks for the "mother" request
            self.counter_left_brother_tasks.fetch_sub(1, Relaxed);
            // we decrease by one the counter of available tasks in the system
            tasks_counter.fetch_sub(1, Relaxed);
        }
    }

    fn is_stealable(&self, tasks_counter: Arc<AtomicUsize>) -> bool {
        // if the elapsed since the "mother" request entered the system
        // is greater than the TARGET_LATENCY...
        if self.request_declaration.elapsed() >= TARGET_LATENCY {
            // ... we decrease the counter of available tasks in the system
            // by self.counter_left_brother_tasks
            tasks_counter.fetch_sub(self.counter_left_brother_tasks.load(Relaxed), Relaxed);
            // we set self.counter_left_brother_tasks to 0 to keep track
            // that we don't want to decrease tasks_counter inside the
            // execute method
            self.counter_left_brother_tasks.store(0, Relaxed);
            false
        } else {
            true
        }
    }
}

fn feed_and_execute(
    global_queue: Shared<VecDeque<Box<dyn Fn() + Send + 'static>>>,
    local_deques: Vec<LocalDeque>,
    eventlogs: Vec<Shared<EventLog>>,
    tasks_counter: Arc<AtomicUsize>,
    local_index: usize,
    termination_receiver: Receiver<()>,
) {
    let mut rng = thread_rng();
    loop {
        // if we get a termination notification we exit the loop
        if termination_receiver.try_recv().is_ok() {
            break;
        }
        // if the local_deque is empty then we accept a new request from
        // the global queue OR we steal a task from another deque
        let local_is_empty = local_deques[local_index].lock().unwrap().is_empty();
        let for_exec_tasks_counter = tasks_counter.clone();
        if local_is_empty {
            // there are tasks in the system that we can steal
            if tasks_counter.load(Relaxed) != 0 {
                let tasks_counter = tasks_counter.clone();
                // we pick a random target
                let mut index;
                loop {
                    index = rng.gen::<usize>() % local_deques.len();
                    if index != local_index {
                        break;
                    }
                }
                let option_task = local_deques[index].lock().unwrap().pop_back();
                if option_task.is_some() {
                    let task = option_task.unwrap();
                    // if the task is stealable we execute it
                    if task.is_stealable(tasks_counter) {
                        task.execute(
                            for_exec_tasks_counter,
                            eventlogs[local_index].clone(),
                            Some(index),
                        );
                    }
                    // otherwise we put it back to its originated deque
                    else {
                        local_deques[index].lock().unwrap().push_back(task);
                    }
                }
            }
            // no more tasks in the system that we can steal so we pick a new request from
            // the global queue
            else {
                // INITIALIZATION : we get the request which only spreads its
                // inner tasks into the deque local to the thread
                let option_request = global_queue.lock().unwrap().pop_front();
                if option_request.is_some() {
                    option_request.unwrap()();
                }
            }
        } else {
            // we get a task from the local_deque and perform it
            let option_task = local_deques[local_index].lock().unwrap().pop_back();
            if option_task.is_some() {
                option_task.unwrap().execute(
                    for_exec_tasks_counter,
                    eventlogs[local_index].clone(),
                    None,
                );
            }
        }
    }
}

struct Threadpool {
    handlers: Vec<JoinHandle<()>>,
    global_queue: Shared<VecDeque<Box<dyn Fn() + Send + 'static>>>,
    tasks_counter: Arc<AtomicUsize>,
    rng: ThreadRng,
    eventlogs: Vec<Shared<EventLog>>,
    termination_sender: Sender<()>,
    time_start: Instant,
}

impl Threadpool {
    fn new(number_of_threads: usize) -> Self {
        // we store the time of declaration of the threadpool
        let time_start = Instant::now();
        // we create a channel which will be used to
        // terminate the threads with the shutdown method
        let (termination_sender, termination_receiver) = bounded(number_of_threads);
        let tasks_counter = Arc::new(AtomicUsize::new(0));
        // we create a global_queue which will holds the
        // requests waiting to be processed
        let global_queue = Arc::new(Mutex::new(VecDeque::new()));
        // we create local deques which will store the children
        // tasks from request, one local deque being assigned
        // to one thread
        let local_deques: Vec<LocalDeque> = (0..number_of_threads)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();
        // we create eventlogs for every local deques + the global
        // queue where we store what is happening during the scenario
        let eventlogs: Vec<Shared<EventLog>> = (0..number_of_threads + 1)
            .map(|_| Arc::new(Mutex::new(EventLog::new())))
            .collect();
        // we create threads which will be our processing units
        let handlers = (0..number_of_threads)
            .map(|i| {
                let global_queue = global_queue.clone();
                let local_deques = local_deques.clone();
                let eventlogs = eventlogs.clone();
                let tasks_counter = tasks_counter.clone();
                let termination_receiver = termination_receiver.clone();
                spawn(move || {
                    // we assign one of local_deques to thread local
                    // local_deque
                    LOCAL_DEQUE.with(|deque| {
                        *deque.borrow_mut() = local_deques[i].clone();
                    });
                    // we assign one of eventlogs to thread local
                    // local eventlog
                    LOCAL_EVENTLOG.with(|eventlog| {
                        *eventlog.borrow_mut() = eventlogs[i].clone();
                    });
                    feed_and_execute(
                        global_queue,
                        local_deques,
                        eventlogs,
                        tasks_counter,
                        i,
                        termination_receiver,
                    )
                })
            })
            .collect();

        Threadpool {
            handlers,
            global_queue,
            tasks_counter,
            rng: thread_rng(),
            eventlogs,
            termination_sender,
            time_start,
        }
    }

    fn forall<T: Fn() + Send + Clone + 'static>(&mut self, repetitions: usize, task: T) {
        let tasks_counter = self.tasks_counter.clone();
        let request_declaration = Instant::now();
        // we create a random color which will help us identify the request
        // in the eventlogs
        let request_color = (self.rng.gen(), self.rng.gen(), self.rng.gen());
        // we store in the eventlog for the global queue (the last one
        // of eventlogs) an AddRequest event
        self.eventlogs[self.eventlogs.len() - 1]
            .lock()
            .unwrap()
            .push(Event {
                category: EventCategory::AddRequest,
                time: request_declaration,
                color: request_color,
            });

        self.global_queue
            .lock()
            .unwrap()
            .push_back(Box::new(move || {
                // we write in the thread local eventlog an event
                // AddTasks
                LOCAL_EVENTLOG.with(|eventlog| {
                    eventlog.borrow_mut().lock().unwrap().push(Event {
                        category: EventCategory::AddTasks(repetitions),
                        time: Instant::now(),
                        color: request_color,
                    });
                });
                // we add to the thread local deque the tasks
                LOCAL_DEQUE.with(|deque| {
                    let counter_left_brother_tasks = Arc::new(AtomicUsize::new(repetitions));
                    for _ in 0..repetitions {
                        deque.borrow_mut().lock().unwrap().push_back(Task::new(
                            task.clone(),
                            request_declaration,
                            counter_left_brother_tasks.clone(),
                            request_color,
                        ));
                    }
                });
                // we increase the counter for number of available
                // tasks in the system
                tasks_counter.fetch_add(repetitions, Relaxed);
            }));
    }

    fn shutdown(self) -> Result<()> {
        // we send a termination notification to every threads
        for _ in 0..self.handlers.len() {
            self.termination_sender
                .send(())
                .expect("Couldn't notify the threads");
        }
        // we join all the threads
        for handler in self.handlers {
            handler.join().expect("Couldn't join the threads");
        }

        // we gather the eventlogs
        let eventlogs: Vec<EventLog> = self
            .eventlogs
            .into_iter()
            .map(|arcmutex| Arc::try_unwrap(arcmutex).unwrap().into_inner().unwrap())
            .collect();

        // we produce the svg displaying the behavior of the threadpool during
        // the scenario
        let mut file = File::create("result.svg")?;
        let (svg_width, svg_height) = (600, 600);
        writeln!(file,"<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\">", svg_width, svg_height, svg_width, svg_height)?;
        display_global_queue(&file, &eventlogs, self.time_start, svg_width, svg_height)?;
        display_local_deques(&file, &eventlogs, self.time_start, svg_width, svg_height)?;
        display_processing_units(&file, &eventlogs, self.time_start, svg_width, svg_height)?;
        writeln!(file, "</svg>")?;

        Ok(())
    }
}

const TARGET_LATENCY: Duration = Duration::from_secs(4);

fn main() {
    let mut threadpool = Threadpool::new(4);
    threadpool.forall(10, || sleep(Duration::from_secs(3)));
    sleep(Duration::from_secs(4));
    threadpool.forall(4, || sleep(Duration::from_secs(2)));
    sleep(Duration::from_secs(15));
    threadpool
        .shutdown()
        .expect("Couldn't create the svg file for the scenario");
}

// on pourrait changer le stroke des tasks quand elles sont plus stealable...
// faire un vrai use case de la threadpool ? article du rustbook la
