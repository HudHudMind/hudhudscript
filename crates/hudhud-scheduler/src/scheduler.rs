//! Cooperative task scheduler for lightweight concurrency.
//!
//! Instead of OS threads per actor, tasks share a thread pool
//! and cooperatively yield at yield points (Yield instruction, await, etc.)

use std::collections::VecDeque;

/// A lightweight task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

/// Task state in the scheduler.
#[derive(Debug)]
pub enum TaskState {
    Ready,
    Running,
    Suspended,
    Completed,
    Failed(String),
}

/// A cooperative task scheduler.
pub struct TaskScheduler {
    ready_queue: VecDeque<TaskId>,
    next_id: u64,
    task_states: std::collections::HashMap<TaskId, TaskState>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            next_id: 1,
            task_states: std::collections::HashMap::new(),
        }
    }

    /// Create a new task and add it to the ready queue.
    pub fn spawn(&mut self) -> TaskId {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        self.task_states.insert(id, TaskState::Ready);
        self.ready_queue.push_back(id);
        id
    }

    /// Get the next ready task (round-robin scheduling).
    pub fn next_ready(&mut self) -> Option<TaskId> {
        while let Some(id) = self.ready_queue.pop_front() {
            if matches!(self.task_states.get(&id), Some(TaskState::Ready)) {
                self.task_states.insert(id, TaskState::Running);
                return Some(id);
            }
        }
        None
    }

    /// Suspend the current task (e.g., on yield or await).
    pub fn suspend(&mut self, id: TaskId) {
        self.task_states.insert(id, TaskState::Suspended);
    }

    /// Resume a suspended task.
    pub fn resume(&mut self, id: TaskId) {
        self.task_states.insert(id, TaskState::Ready);
        self.ready_queue.push_back(id);
    }

    /// Mark a task as completed.
    pub fn complete(&mut self, id: TaskId) {
        self.task_states.insert(id, TaskState::Completed);
    }

    /// Mark a task as failed.
    pub fn fail(&mut self, id: TaskId, error: String) {
        self.task_states.insert(id, TaskState::Failed(error));
    }

    /// Number of active (non-completed) tasks.
    pub fn active_count(&self) -> usize {
        self.task_states
            .values()
            .filter(|s| !matches!(s, TaskState::Completed | TaskState::Failed(_)))
            .count()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
