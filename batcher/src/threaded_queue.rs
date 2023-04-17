use std::fmt::Display;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct Processor
{
    thread_count: i32
}

impl<'a> Processor
{
    pub fn new(thread_count: i32) -> Processor
    {
        Processor { thread_count }
    }

    pub fn setup_queue<T: std::clone::Clone + Send + Display + 'static>(&mut self, array: &[T], callback: impl Fn(T) + Send + Sync + 'static)
    {
        let mut thread_handles = Vec::with_capacity(self.thread_count as usize);
        let vec_arcmut = Arc::new(Mutex::new(Vec::from(array)));
        let callback_arcmut = Arc::new(Box::new(callback));
        
        for index in 0..self.thread_count
        {
            let local_callback_arcmut = Arc::clone(&callback_arcmut);
            let local_vec_arcmut = Arc::clone(&vec_arcmut);
            thread_handles.push(Box::new(thread::spawn(move || {
                let id = index + 1;
                println!("Thread {id} starting.");
                loop
                {
                    println!("[{id}] Waiting to acquire lock");
                    let mut lock = local_vec_arcmut.lock().unwrap();
                    println!("[{id}] Acquired lock.");
                    let element = match lock.pop()
                    {
                        Some(x) => x,
                        None =>
                        {
                            eprintln!("[{index}] Vec is empty.");
                            break;
                        }
                    };
                    let size = lock.len();
                    drop(lock);
                    
                    println!("[{}] Consuming {}", index + 1, element);
                    _ = local_callback_arcmut(element);

                    if size == 0 // Prevents from doing another loop.
                    {
                        break;
                    }
                }
                println!("[{index}] Finished working.");
            })));
        }

        println!("Waiting for threads 1..{} to finish...", thread_handles.len());
        thread_handles.reverse();
        while let Some(handle) = thread_handles.pop()
        {
            _ = match handle.join()
            {
                Ok(_) => continue,
                Err(_) => eprintln!("Failed to join handle.")
            }
        }
    }
}