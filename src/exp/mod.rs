use std::collections::VecDeque;
use std::default;
use std::ops::DerefMut;
/// Experimental nonsense
use std::sync::{Arc, Mutex};

pub trait JobTrait: Send {
    fn run(&mut self);
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum JobStatus {
    Running,
    #[default]
    Waiting,
    Quit,
}

use std::sync::atomic::AtomicBool;

pub struct Worker {
    idx: usize,
    status: Mutex<JobStatus>,
    q: Mutex<VecDeque<Box<dyn JobTrait>>>,
    please_die: AtomicBool,

    glob_jobs: Arc<Mutex<Vec<Box<dyn JobTrait>>>>,
    workers: Arc<Mutex<Vec<Worker>>>,
}

struct PoolData {
    jobs: Vec<Box<dyn JobTrait>>,
    workers: Vec<Worker>,
}

unsafe impl Send for Worker {}

impl Worker {
    pub fn new(
        idx: usize,
        glob_jobs: Arc<Mutex<Vec<Box<dyn JobTrait>>>>,
        workers: Arc<Mutex<Vec<Worker>>>,
    ) -> Self {
        Self {
            status: Default::default(),
            q: Default::default(),
            idx,
            please_die: false.into(),
            glob_jobs,
            workers,
        }
    }

    fn status(&self) -> JobStatus {
        *self.status.lock().unwrap()
    }

    fn set_status(&self, j: JobStatus) {
        *self.status.lock().unwrap() = j;
    }

    fn get_job(&self) -> Option<Box<dyn JobTrait>> {
        println!("Trying to get a job!");
        self.get_pending_job()
            .or_else(|| self.get_job_from_pool())
            .or_else(|| self.steal_job_from_workers())
    }

    fn get_pending_job(&self) -> Option<Box<dyn JobTrait>> {
        let ret = self.q.lock().unwrap().pop_front();
        println!("Trying to get one of my own jobs {}", ret.is_some());
        ret
    }

    fn get_job_from_pool(&self) -> Option<Box<dyn JobTrait>> {
        let mut glob_jobs = self.glob_jobs.lock().ok()?;
        let ret = glob_jobs.pop();
        println!("Trying to get a pool job {}", ret.is_some());
        ret
    }

    fn steal_job_from_workers(&self) -> Option<Box<dyn JobTrait>> {
        println!("Trying to steal jobs!");
        let num_of_workers = { self.workers.lock().unwrap().len() };

        // Traverse the vec of workers
        // start index of next job
        for i in (self.idx + 1)..(self.idx + num_of_workers) {
            let idx = i % num_of_workers;
            let job = self.workers.try_lock().ok().and_then(|w| w[idx].steal());
            if job.is_some() {
                return job;
            }
        }
        println!("None");

        None
    }

    pub fn wait(&self) {
        // this should be implmented with a condvar
        std::thread::yield_now()
    }

    pub fn run(&self) {
        println!("Starting thread {}", self.idx);

        while !self.please_die.load(std::sync::atomic::Ordering::Relaxed) {
            println!("Deciding what to do");

            if let Some(mut job) = self.get_job() {
                println!("Got a job!");
                self.set_status(JobStatus::Running);
                job.run()
            } else {
                println!("Waiting for a job!");
                self.set_status(JobStatus::Waiting);
            }

            println!("yielding");
            self.wait();
        }

        self.set_status(JobStatus::Quit);
    }

    pub fn join(&self) {
        self.please_die
            .store(true, std::sync::atomic::Ordering::Relaxed);

        while self.status() != JobStatus::Quit {
            std::thread::yield_now()
        }
    }

    pub fn add(&self, job: Box<dyn JobTrait>) {
        self.q.lock().unwrap().push_front(job)
    }

    pub fn steal(&self) -> Option<Box<dyn JobTrait>> {
        self.q.lock().unwrap().pop_back()
    }
}

pub struct Pool<const N: usize> {
    workers: Arc<Mutex<Vec<Worker>>>,
    jobs: Arc<Mutex<Vec<Box<dyn JobTrait>>>>,
    pool_data: Arc<Mutex<PoolData>>,
}

impl<const N: usize> Pool<N> {
    pub fn add<J: JobTrait + 'static>(&mut self, job: J) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(Box::new(job))
    }

    pub fn pool_jobs_remaining(&self) -> usize {
        let jobs = self.jobs.lock().unwrap();
        jobs.len()
    }

    pub fn worker_jobs_remaining(&self) -> usize {
        self.workers
            .lock()
            .unwrap()
            .iter()
            .fold(0, |t, w| t + w.q.lock().unwrap().len())
    }

    pub fn join(&self) {
        // wait for all jobs to empty
        loop {
            if self.pool_jobs_remaining() == 0 {
                break;
            } else {
                std::thread::yield_now();
            }
        }

        loop {
            if self.worker_jobs_remaining() != 0 {
                std::thread::yield_now()
            } else {
                {
                    for w in self.workers.lock().unwrap().iter() {
                        w.please_die
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                std::thread::yield_now()
            }

            if self
                .workers
                .lock()
                .unwrap()
                .iter()
                .all(|w| w.status() == JobStatus::Quit)
            {
                break;
            } else {
                std::thread::yield_now()
            }
        }
    }

    fn add_worker_thread(&self) {
        let _idx = {
            let idx = self.workers.lock().unwrap().len();
            assert!(idx < N);
            let worker = Worker::new(idx, self.jobs.clone(), self.workers.clone());
            let mut w = self.workers.lock().unwrap();
            w.push(worker);
            idx
        };

        let workers = self.workers.clone();

        let _id = std::thread::spawn(move || loop {
            let _w = workers.lock().unwrap();
            _w[_idx].run()
        });
    }
    pub fn new() -> Self {

        let pool_data = PoolData {
            jobs: vec![],
            workers: vec![],
        };

        let pool_data = Arc::new(Mutex::new(pool_data));

        let jobs = Default::default();
        let workers = Mutex::new(Vec::with_capacity(N)).into();

        let ret = Self {
            jobs,
            workers,
            pool_data,
        };

        for _ in 0..N {
            ret.add_worker_thread()
        }
        ret
    }
}

pub mod test {
    use super::{JobTrait, Pool};
    use rand::Rng;
    use std::time::Duration;

    #[derive(Debug)]
    struct FakeWork {
        wait_millis: usize,
        idx: usize,
    }

    impl JobTrait for FakeWork {
        fn run(&mut self) {
            println!(
                "[{}] starting, waiting for {}ms",
                self.idx, self.wait_millis
            );
            std::thread::sleep(Duration::from_millis(self.wait_millis as u64));
            println!("[{}] stopping", self.idx);
        }
    }

    impl FakeWork {
        pub fn new(idx: usize) -> Self {
            let mut rng = rand::thread_rng();
            Self {
                idx,
                wait_millis: rng.gen_range(150..1000),
            }
        }
    }

    pub fn test_pool() {
        println!("Creating pool");
        let mut pool = Pool::<10>::new();
        println!("Pool created");

        for idx in 0..30 {
            let work = FakeWork::new(idx);
            println!("Adding working for pool {idx}");
            pool.add(work)
        }

        println!("joining pool");
        pool.join();
    }
}
