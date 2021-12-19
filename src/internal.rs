use std::{sync::{Arc, Mutex, mpsc}, thread};

type Task = Box<dyn FnOnce() + Send + 'static>;

enum Order {
    Do(Task),
    Stop
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, cin: Arc<Mutex<mpsc::Receiver<Order>>>) -> Worker {
        let thread = Some(thread::spawn(move|| loop {
            let order = cin
                .lock().unwrap()
                .recv().unwrap();

            match order {
                Order::Do(task) => task(),
                Order::Stop => break 
            };
        }));

        Worker{id, thread}
    }
}

pub struct Threadpool {
    size: usize,
    cout: mpsc::Sender<Order>,
    workers: Vec<Worker>
}

#[allow(dead_code)]
impl Threadpool {
    pub fn new(size: usize) -> Threadpool {

        let (cout, cin) = mpsc::channel::<Order>();
        let cin = Arc::new(Mutex::new(cin));

        let mut workers = Vec::<Worker>::with_capacity(size);
        for id in 0..size { workers.push(Worker::new(id, Arc::clone(&cin))); }

        Threadpool{size, cout, workers}
    }

    pub fn execute<F>(&self, f: F) 
        where
            F: FnOnce() + Send + 'static
    {
        self.cout
            .send(Order::Do(Box::new(f)))
            .unwrap();
    }

    pub fn get_size(&self) -> usize { self.size }
    pub fn get_all_workerid(&self) -> Vec<usize> { 
        self.workers.iter()
            .map(|w| w.id)
            .collect() 
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        self.workers
            .iter()
            .for_each(|_| 
                self.cout.send(Order::Stop).unwrap()
            );

        self.workers
            .iter_mut()
            .for_each(|worker|
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            );

        // for worker in &mut self.workers {
        //     if let Some(thread) = worker.thread.take() {
        //         thread.join().unwrap();
        //     }
        // }
    }
}