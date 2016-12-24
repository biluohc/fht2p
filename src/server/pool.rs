#![allow(dead_code)]
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{Ordering, AtomicUsize};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::Duration;
use std::error::Error;
use std::thread;

// 最低线程数
const MIN_THREADS: usize = 4;
// 线程栈大小 4M
const THREAD_STACK_SIZE: usize = 4 * 1024 * 1024;
// 线程销毁时间 ms
const THREAD_TIME_OUT_MS: u64 = 5000;

pub struct Pool {
    water: Arc<Water>,
}

struct Water {
    tasks: Mutex<VecDeque<Box<FnMut() + Send>>>,
    condvar: Condvar,
    threads: AtomicUsize,
    threads_waited: AtomicUsize,
}

impl Pool {
    pub fn new() -> Pool {
        let pool = Pool {
            water: Arc::new(Water {
                tasks: Mutex::new(VecDeque::new()),
                condvar: Condvar::new(),
                threads: AtomicUsize::new(0),
                threads_waited: AtomicUsize::new(0),
            }),
        };

        for _ in 0..MIN_THREADS {
            pool.add_thread();
        }

        pool
    }

    pub fn spawn(&self, task: Box<FnMut() + Send>) {
        let mut tasks_queue = self.water.tasks.lock().unwrap();
        // {
        //     println!("\nPool_waits/threads: {}/{} ---tasks_queue:  {}",
        //              (&self.water.threads_waited).load(Ordering::Acquire),
        //              (&self.water.threads).load(Ordering::Acquire),
        //              tasks_queue.len());
        // }
        if (&self.water.threads_waited).load(Ordering::Acquire) == 0 {
            self.add_thread();
        } else {
            self.water.condvar.notify_one();
        }
        tasks_queue.push_back(task);
    }

    fn add_thread(&self) {
        let water = self.water.clone();
        // spawn 有延迟,必须等父线程阻塞才运行.
        let spawn_res = thread::Builder::new()
            .stack_size(THREAD_STACK_SIZE)
            .spawn(move || {
                let water = water;
                // 对线程计数.
                let _threads_counter = Counter::new(&water.threads);

                loop {
                    let mut tasks = water.tasks.lock().unwrap();//取得锁

                    let mut task;
                    loop {
                        if let Some(poped_task) = tasks.pop_front() {
                            task = poped_task;// pop成功就break ,执行pop出的任务.
                            break;
                        }
                        // 对在等候的线程计数.
                        let _threads_waited_counter = Counter::new(&water.threads_waited);

                        match (&water.threads).load(Ordering::Acquire) {
                            0...MIN_THREADS => tasks = water.condvar.wait(tasks).unwrap(), //线程总数<最小限制,不销毁线程.
                            _ => {
                                let (new_tasks, waitres) = water.condvar
                                    .wait_timeout(tasks, Duration::from_millis(THREAD_TIME_OUT_MS))
                                    .unwrap();
                                tasks = new_tasks;
                               // timed_out()为true时(等待超时是收不到通知就知道超时), 且队列空时销毁线程。
                                if waitres.timed_out() &&tasks.is_empty(){
                                return;//销毁线程。
                            }
                        }
                    }; // match 结束。
                    } // loop 结束。
                    task();//执行任务。
                } // loop 结束。
            }); //spawn 结束。

        match spawn_res {
            Ok(_) => {}
            Err(e) => {
                std_err(&format!("Warnig:spawn failed because of {} !", e.description()));
            }
        };
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        // 被唤醒的线程,如果线程总数>线程最小限制就会陷入waited_out,然后线程销毁.
        self.water.threads.store(usize::max_value(), Ordering::Release);
        self.water.condvar.notify_all();
    }
}

// 通过作用域对线程数目计数。
struct Counter<'a> {
    count: &'a AtomicUsize,
}

impl<'a> Counter<'a> {
    fn new(count: &'a AtomicUsize) -> Counter<'a> {
        count.fetch_add(1, Ordering::Release);
        Counter { count: count }
    }
}

impl<'a> Drop for Counter<'a> {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::Release);
    }
}
// 格式化标准错误输出
fn std_err(msg: &str) {
    match writeln!(io::stderr(), "{}", msg) {    
        Ok(..) => {}
        Err(e) => panic!("{}\n{}\n", msg, e.description()),
    };
}

fn main() {
    println!("Hello, world!");
    let pool = Pool::new();

    let mut count = 0;
    loop {
        if count == 100 {
            break;
        }
        for i in 0..32 {
            print!("main_loop0: ");
            pool.spawn(Box::new(move || test(count, i)));
        }
        thread::sleep(Duration::from_millis(1));
        count += 1;
    }
    count = 0;
    thread::sleep(Duration::from_millis(6000));
    println!("main_loop0 finished: ");
    loop {
        if count == 100 {
            break;
        }
        for i in 0..20 {
            print!("main_loop1: ");
            pool.spawn(Box::new(move || test(count, i)));
        }
        thread::sleep(Duration::from_millis(100));
        count += 1;
    }
    println!("loop1 finished ! Running a fib(20)");
    pool.spawn(Box::new(move || test(count, 20)));
    fn test(count: i32, msg: i32) {
        println!("count({})_fib({})={}", count, msg, fib(msg));
    }
    fn fib(msg: i32) -> i32 {
        match msg {
            0...2 => return 1,
            x @ _ => return fib(x - 1) + fib(x - 2),
        };
    }
}