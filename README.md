A schemey lisp

# Function calling notes
* Each function needs an activation record
* Details the function's storage needs
    * Parameters
    * Temporaries
    * Results
* Points to calling function record
    * Not sure why? Exceptions?
* Need to keep in mind dynamic scoping vs lexical scoping
    * Enclosing scopes are different

# Register Allocation Notes
* Liveness analysis
    * Interference graph
        * Constructed by understanding what registers are alive at any given point
        * Have to choose an appropriate scope - function? Lexical scope? Nested?
    * Graph coloring
        * Color graph with N colours, N == amount of registers 
        * Not all registers can be colored per gc rules
        * Uncolored registers are 'spilled' - relegated to stack storage
        * BUT the decision to spill needs to be weighted according to the cost of stacking the register


# Job Stealing Q - lol
* A worker thread has
    * a local q
    * a reference to a global q
    * a way to try and steal work of any other queues
* A threadpool has
    * A number of worker threads
    * A global q of work to do
    * A thread? Maybe

* A thread pool can
    * Receive work
    * Join until all threads are done
    * Provide info on what's being done

* A thread's local work q is
    * A deque
    * Can pop work from the front
    * Can have work stolen from the end

* When stealing from other worker threads
    * Try the next thread index and keep trying until all others are done
    * Don't have to check if this is your index
    * Also! Stops the first worker thread from being continually polled for spare work
        * Mutex contention

* How would a thread pool receive work?
    * Maybe a dyn ptr to a struct that implements a job trait

```rust 
pub enum JobStatus {
    Waiting,
    Running,
}

pub trait JobTrait {
    fn run(&mut self);
}

pub struct Pool {
}

pub struct Worker {
}

impl Worker {
}

pub struct Queue {
    q : Mutex<Deque<Box<JobTrait>>>,
}

impl Queue {
    pub fn steal(&mut self) -> Option<Box<JobTrait>> {
        let q = self.q.unlock()?;
        q.pop_back()
    }

    pub fn pop(&mut self) -> Option<dyn JobTrait> {
        let q = self.q.unlock()?;
        q.pop_front()
    }
}

```


