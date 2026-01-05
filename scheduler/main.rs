use std::collections::{HashMap, VecDeque};
use std::num::{NonZero, NonZeroUsize};

// =========================================================================
// PARTEA 1: DEFINIȚIILE (Reconstruite din cerința temei)
// =========================================================================

pub type Pid = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Waiting,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub state: ProcessState,
    pub timings: (u128, u128, u128), // Simplificat pentru test
}

#[derive(Debug)]
pub enum SchedulingDecision {
    Run { pid: Pid, timeslice: NonZeroUsize },
    Sleep(NonZeroUsize),
    Deadlock,
    Panic,
    Done,
}

#[derive(Debug)]
pub enum Syscall {
    Fork(i8),           // prioritate
    Sleep(NonZeroUsize),
    Exit,
    Wait(usize),        // event number
    Signal(usize),      // event number
}

#[derive(Debug)]
pub enum StopReason {
    Syscall { syscall: Syscall, remaining: usize, pid: Pid },
    Expired { pid: Pid },
}

#[derive(Debug)]
pub enum SyscallResult {
    Pid(Pid),
    Success,
    NoRunningProcess,
}

// Aceasta este interfața pe care trebuie să o implementezi
pub trait Scheduler {
    fn next(&mut self) -> SchedulingDecision;
    fn stop(&mut self, reason: StopReason) -> SyscallResult;
    fn list(&mut self) -> Vec<ProcessInfo>;
}


#[derive(Debug, Clone)]
struct CfsProcess {
    pid: Pid,
    state: ProcessState,
    vruntime: u128, // Contorul de timp executat
}

// Implementăm compararea doar pe baza vruntime-ului
use std::cmp::Ordering;
use std::os::unix::process;

impl PartialEq for CfsProcess {
    fn eq(&self, other: &Self) -> bool {
        self.vruntime == other.vruntime
    }
}

impl PartialOrd for CfsProcess {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Vrem ca cel cu vruntime MIC să fie considerat "mai mare" (prioritar) 
        // sau pur și simplu le comparăm natural:
        self.vruntime.partial_cmp(&other.vruntime)
    }
}


// =========================================================================
// PARTEA 2: IMPLEMENTAREA TA (Round Robin)
// =========================================================================

struct MyProcess {
    pid: Pid,
    state: ProcessState,
    priority: i8,
    // Aici am putea stoca cât mai are de dormit etc.
}

pub struct RoundRobin {
    processes: HashMap<Pid, MyProcess>,
    queue: VecDeque<Pid>,
    timeslice: NonZeroUsize,
    next_pid: Pid, // Counter pentru a genera PID-uri noi
}

pub struct RobinPriority {
    processes: HashMap<Pid, MyProcess>,
    queues: [VecDeque<Pid>; 6],
    timeslice: NonZeroUsize,
    next_pid: Pid,
}

impl RoundRobin {
    pub fn new(timeslice: NonZeroUsize) -> Self {
        Self {
            processes: HashMap::new(),
            queue: VecDeque::new(),
            timeslice,
            next_pid: 1, 
        }
    }
}

impl RobinPriority {
    pub fn new(timeslice :  NonZeroUsize) -> Self {
        let queues : [VecDeque<Pid>; 6] = Default::default();
        Self {
            processes : HashMap :: new(),
            queues,
            timeslice :  timeslice,
            next_pid : 1
        }
    }
}
pub struct CfsScheduler {
    processes: HashMap<Pid, CfsProcess>,
    cfs_base_time: NonZeroUsize, // De ex: 20
    next_pid: Pid,
}

impl CfsScheduler {
    pub fn new(base_time: NonZeroUsize) -> Self {
        Self {
            processes: HashMap::new(),
            cfs_base_time: base_time, 
            next_pid: 1, 
        }
    }
}

impl Scheduler for RoundRobin {
    fn next(&mut self) -> SchedulingDecision {
        if let Some(pid) = self.queue.pop_front() {
            if let Some(proc) = self.processes.get_mut(&pid) {
                proc.state = ProcessState::Running;

                return SchedulingDecision::Run { 
                    pid, 
                    timeslice: self.timeslice };
            };
        }

        if self.queue.is_empty() {
            return SchedulingDecision::Sleep(NonZeroUsize::new(1).unwrap());
        }

        SchedulingDecision::Done
    }

    fn stop(&mut self, reason: StopReason) -> SyscallResult {
        match reason {
            StopReason::Expired { pid } => {
                if let Some(proc) = self.processes.get_mut(&pid) {
                    proc.state = ProcessState::Ready;
                    self.queue.push_back(pid);
                }
                return SyscallResult::Success;
            },
            StopReason::Syscall { syscall, remaining, pid } => {
                match syscall {
                    Syscall::Fork(prio) => {
                        let child_pid = self.next_pid;
                        self.next_pid += 1;
                        let new_proc = MyProcess {
                            pid : child_pid,
                            state : ProcessState::Ready,
                            priority : prio,
                        };
                        self.processes.insert(child_pid, new_proc);
                        self.queue.push_back(child_pid);
                        if let Some(parent) = self.processes.get_mut(&pid) {
                            parent.state = ProcessState::Ready;
                            self.queue.push_back(pid);
                        }
                        return SyscallResult::Pid(child_pid);
                    },
                    Syscall::Exit => {
                        self.processes.remove(&pid);
                        return SyscallResult::Success;
                    }
                    _ => {
                        if let Some(proc) = self.processes.get_mut(&pid) {
                            proc.state = ProcessState::Ready;
                            self.queue.push_back(pid);
                        }
                        return SyscallResult::Success;
                    }
                }
            }
        }
    }

    fn list(&mut self) -> Vec<ProcessInfo> {
        self.processes.values().map( |p| ProcessInfo {
            pid : p.pid,
            state : p.state.clone(),
            timings : (0,0,0,)
        }).collect()
    }
}

impl Scheduler for RobinPriority {
    fn next(&mut self) -> SchedulingDecision {
        for i in (0..6).rev() {
            if !self.queues[i].is_empty() {
                if let Some(pid) = self.queues[i].pop_front() {
                    if let Some(proc) = self.processes.get_mut(&pid) {
                        proc.state = ProcessState::Running;
                        return SchedulingDecision::Run { pid, timeslice: (self.timeslice) };
                    }

                }
            }
        }  
        return SchedulingDecision::Sleep(NonZeroUsize::new(1).unwrap());
    }

    fn stop(&mut self, reason: StopReason) -> SyscallResult {
        match reason {
            StopReason::Expired { pid } => {
                if let Some(proc) = self.processes.get_mut(&pid) {
                    if proc.priority > 0 {
                        proc.priority -= 1;
                    }
                    proc.state = ProcessState::Ready;
                    self.queues[proc.priority as usize].push_back(pid);
                }
                return SyscallResult::Success;
            }
            StopReason::Syscall { syscall, remaining, pid } => {
                match syscall {
                    Syscall::Exit => {
                        self.processes.remove(&pid);
                        return SyscallResult::Success;
                    }
                    Syscall::Fork(prio) => {
                            let childProc =  MyProcess {
                                pid : self.next_pid,
                                state : ProcessState::Ready,
                                priority : prio
                            };
                            self.next_pid += 1;
                            self.queues[childProc.priority as usize].push_back(childProc.pid);
                            let kpid = childProc.pid;
                            self.processes.insert(childProc.pid, childProc);   
                            if let Some(proc) = self.processes.get_mut(&pid) {
                                if proc.priority < 5 {
                                    proc.priority += 1;
                                }
                                proc.state = ProcessState::Ready;
                                self.queues[proc.priority as usize].push_back(pid);
                            }
                            return  SyscallResult::Pid(kpid);
                    }
                    _ => {
                        if let Some(proc) = self.processes.get_mut(&pid) {
                            if (proc.priority < 5) {
                                    proc.priority += 1;
                            }
                            proc.state = ProcessState::Ready;
                            self.queues[proc.priority as usize].push_back(pid);
                        }
                        return SyscallResult::Success;
                    }
                }
            }
        }
    }

    fn list(&mut self) -> Vec<ProcessInfo> {
        self.processes.values().map(|p| ProcessInfo {
            pid : p.pid,
            state : p.state.clone(),
            timings : (0,0,0)
        }).collect()
    }
}

impl Scheduler for CfsScheduler {
    fn next(&mut self) -> SchedulingDecision {
        let ready_pid : Vec<Pid> = self.processes
        .iter()
        .filter(|(_,p)| p.state == ProcessState::Ready)
        .map(|(pid, _)| *pid)
        .collect();

        if ready_pid.len() == 0 {
            if self.processes.is_empty() {
                return SchedulingDecision::Done;
            } else {
                return SchedulingDecision::Sleep(NonZeroUsize::new(1).unwrap());
            }
        }

        let base = self.cfs_base_time.get();
        let slice_clc = base / ready_pid.len();
        let final_slice = if slice_clc < 1 {1} else {slice_clc};
        let timeslice = NonZeroUsize :: new(final_slice).unwrap();

        let mut min_vruntime = u128::max_value();
        let mut s_pid = 0;

        for pid in ready_pid {
            if let Some(proc) = self.processes.get_mut(&pid) {
                if proc.vruntime < min_vruntime {
                    min_vruntime = proc.vruntime;
                    s_pid = pid;
                }
            }
        }

        if let Some(proc) = self.processes.get_mut(&s_pid) {
            proc.state = ProcessState::Running;
            
        }

        return SchedulingDecision::Run {
        pid: s_pid,
        timeslice,
        };

    }

    fn stop(&mut self, reason: StopReason) -> SyscallResult {

        let count_ready = self.processes
        .iter().
        filter(|(_, proc)| proc.state != ProcessState::Waiting)
        .count();

        let base = self.cfs_base_time.get();
        let active_procs = self.processes.iter()
        .filter(|(_, p)| p.state == ProcessState::Ready || p.state == ProcessState::Running)
        .count();

        let safe_count = if active_procs == 0 {1} else {active_procs};

        let slice_calc = base / safe_count;
        let allocated_time = if slice_calc < 1 { 1 } else { slice_calc };

        match reason {
            StopReason::Expired { pid  } => {
                if let Some(proc) = self.processes.get_mut(&pid) {
                    proc.vruntime += allocated_time as u128;
                    proc.state = ProcessState::Ready;
                }
                return SyscallResult::Success;
            }
            StopReason::Syscall { syscall, remaining, pid } => {
                let executed = allocated_time - remaining;
                if let Some(proc) = self.processes.get_mut(&pid) {
                    proc.vruntime += executed as u128;
                }
                match syscall {
                    Syscall::Exit =>{
                        self.processes.remove(&pid);
                        return SyscallResult::Success;
                    }
                    Syscall::Fork(prio) => {
                        let parent_runtime = self.processes.get_mut(&pid).map(|p|p.vruntime).unwrap_or(0);
                        let k_process = CfsProcess {
                            pid : self.next_pid,
                            state : ProcessState::Ready,
                            vruntime : parent_runtime
                        };
                        self.next_pid += 1;
                        let k_pid = k_process.pid;
                        self.processes.insert(k_pid, k_process);
                        if let Some(proc) = self.processes.get_mut(&pid) {
                            proc.state = ProcessState::Ready;
                        }
                        return SyscallResult::Pid(k_pid);
                    }
                    _ => {
                        if let Some(proc) = self.processes.get_mut(&pid) {
                            proc.state = ProcessState::Ready;
                        }
                        return SyscallResult::Success;
                    }
                }
            }
        }
    }

    fn list(&mut self) -> Vec<ProcessInfo> {
        self.processes
        .values()
        .map(|p|  ProcessInfo{
            pid : p.pid,
            state : p.state.clone(),
            timings : (0,0,0)
        }).collect()
    }
}

// =========================================================================
// PARTEA 3: SIMULATORUL (Scenariul de test)
// =========================================================================

fn main() {
    // 1. Definim timpul de bază pentru CFS
    let base_time = NonZeroUsize::new(20).unwrap(); // Timp total mai mare, ca să se împartă
    
    // 2. Inițializăm CFS Scheduler (NU RobinPriority)
    println!("=== TEST CFS SCHEDULER ===");
    let mut scheduler = CfsScheduler::new(base_time);

    println!("--- 1. Initializare: Sistemul porneste ---");
    
    // PID 0 crează PID 1
    println!("[Simulator] Trimitem primul FORK...");
    let result = scheduler.stop(StopReason::Syscall {
        syscall: Syscall::Fork(0),
        remaining: 0,
        pid: 0, 
    });
    println!("[Simulator] Rezultat Fork initial: {:?}", result);

    println!("\n--- 2. Incepem bucla de executie ---");

    // Simulăm mai mulți pași
    for pas in 1..=15 {
        println!("\n>> PASUL {}", pas);
        
        // 1. NEXT
        let decision = scheduler.next();
        println!("[Scheduler] Decizie: {:?}", decision);

        match decision {
            SchedulingDecision::Run { pid, timeslice } => {
                println!("[CPU] Ruleaza PID {} cu timeslice {}", pid, timeslice);
                
                // Scenariu:
                // Pas 2: PID 1 face Fork -> Apare PID 2
                // Pas 5: PID 2 face Fork -> Apare PID 3
                // Pas 10: PID 1 face Exit
                
                if pas == 2 && pid == 1 {
                    println!("      -> Face FORK!");
                    scheduler.stop(StopReason::Syscall {
                        syscall: Syscall::Fork(0),
                        remaining: timeslice.get() - 2, // A consumat 2 unități
                        pid,
                    });
                } else if pas == 5 && pid == 2 {
                    println!("      -> Face FORK (PID 2 face copil)!");
                    scheduler.stop(StopReason::Syscall {
                        syscall: Syscall::Fork(0),
                        remaining: timeslice.get() - 1,
                        pid,
                    });
                } else if pas == 10 && pid == 1 {
                    println!("      -> Face EXIT!");
                    scheduler.stop(StopReason::Syscall {
                        syscall: Syscall::Exit,
                        remaining: 1,
                        pid,
                    });
                } else {
                    println!("      -> Expired (Consuma tot timpul)");
                    scheduler.stop(StopReason::Expired { pid });
                }
            },
            SchedulingDecision::Done => {
                println!("[Simulator] Gata!");
                break;
            }
            _ => { println!("Waiting..."); }
        }
    }
    
    println!("\n--- Stare finala (Vruntime check) ---");
    // Nu putem vedea vruntime prin list() standard, dar vedem PID-urile rămase
    for p in scheduler.list() {
        println!("PID: {}, Stare: {:?}", p.pid, p.state);
    }
}