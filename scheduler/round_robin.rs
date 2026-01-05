pub struct RoundRobin {
    process_map : HashMap<Pid, MyProcessData>,
    vec_pid : VecDeque<Pid>
}