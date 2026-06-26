//! Rjango Tasks — Django 6.0 background tasks system.
//!
//! Provides a task queue, worker, and registry for running
//! background/async tasks, modeled after Django's new task framework.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Represents the status of a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "queued" => TaskStatus::Queued,
            "running" => TaskStatus::Running,
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Queued,
        }
    }
}

/// The result of executing a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl TaskResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }
}

/// A task in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub args: HashMap<String, String>,
    pub status: TaskStatus,
    pub created_at: String,
    pub scheduled_at: Option<String>,
    pub result: Option<TaskResult>,
}

impl Task {
    pub fn new(name: impl Into<String>, args: HashMap<String, String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            args,
            status: TaskStatus::Queued,
            created_at: chrono::Utc::now().to_rfc3339(),
            scheduled_at: None,
            result: None,
        }
    }

    pub fn scheduled(name: impl Into<String>, args: HashMap<String, String>, at: impl Into<String>) -> Self {
        let mut task = Self::new(name, args);
        task.scheduled_at = Some(at.into());
        task
    }
}

/// A handle for tracking and managing an enqueued task.
#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: String,
    queue: Arc<Mutex<dyn TaskQueue + Send>>,
}

impl TaskHandle {
    pub fn new(id: String, queue: Arc<Mutex<dyn TaskQueue + Send>>) -> Self {
        Self { id, queue }
    }

    pub fn status(&self) -> TaskStatus {
        self.queue.lock().ok()
            .and_then(|q| q.get_task(&self.id))
            .map(|t| t.status)
            .unwrap_or(TaskStatus::Cancelled)
    }

    pub fn cancel(&self) -> bool {
        self.queue.lock().ok()
            .map(|mut q| q.cancel(&self.id))
            .unwrap_or(false)
    }
}

/// Task function signature.
pub type TaskFn = Arc<dyn Fn(HashMap<String, String>) -> TaskResult + Send + Sync>;

/// Definition of a registered task.
#[derive(Clone)]
pub struct TaskDef {
    pub name: String,
    pub function: TaskFn,
    pub description: String,
}

impl std::fmt::Debug for TaskDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskDef")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

impl TaskDef {
    pub fn new(name: impl Into<String>, function: TaskFn, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            function,
            description: description.into(),
        }
    }

    pub fn execute(&self, args: HashMap<String, String>) -> TaskResult {
        (self.function)(args)
    }
}

/// Registry of available tasks.
#[derive(Default, Debug)]
pub struct TaskRegistry {
    tasks: Mutex<HashMap<String, TaskDef>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, def: TaskDef) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(def.name.clone(), def);
    }

    pub fn get(&self, name: &str) -> Option<TaskDef> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(name).cloned()
    }

    pub fn all(&self) -> Vec<TaskDef> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        let tasks = self.tasks.lock().unwrap();
        tasks.contains_key(name)
    }
}

/// A singleton global task registry.
pub fn global_registry() -> &'static TaskRegistry {
    static REGISTRY: std::sync::LazyLock<TaskRegistry> = std::sync::LazyLock::new(TaskRegistry::new);
    &REGISTRY
}

/// Trait for task queue backends.
pub trait TaskQueue: Send + std::fmt::Debug {
    fn enqueue(&mut self, task: Task) -> String;
    fn dequeue(&mut self) -> Option<Task>;
    fn get_task(&self, id: &str) -> Option<Task>;
    fn cancel(&mut self, id: &str) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn all(&self) -> Vec<Task>;
    fn failed(&self) -> Vec<Task>;
    fn completed(&self) -> Vec<Task>;
}

/// In-memory task queue backed by VecDeque.
#[derive(Debug)]
pub struct MemoryTaskQueue {
    tasks: HashMap<String, Task>,
    queue: std::collections::VecDeque<String>,
}

impl MemoryTaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            queue: std::collections::VecDeque::new(),
        }
    }
}

impl TaskQueue for MemoryTaskQueue {
    fn enqueue(&mut self, task: Task) -> String {
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        self.queue.push_back(id.clone());
        id
    }

    fn dequeue(&mut self) -> Option<Task> {
        while let Some(id) = self.queue.pop_front() {
            if let Some(task) = self.tasks.get_mut(&id) {
                if task.status == TaskStatus::Queued {
                    task.status = TaskStatus::Running;
                    return Some(task.clone());
                }
            }
        }
        None
    }

    fn get_task(&self, id: &str) -> Option<Task> {
        self.tasks.get(id).cloned()
    }

    fn cancel(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(id) {
            if task.status == TaskStatus::Queued {
                task.status = TaskStatus::Cancelled;
                return true;
            }
        }
        false
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn all(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    fn failed(&self) -> Vec<Task> {
        self.tasks.values().filter(|t| t.status == TaskStatus::Failed).cloned().collect()
    }

    fn completed(&self) -> Vec<Task> {
        self.tasks.values().filter(|t| t.status == TaskStatus::Completed).cloned().collect()
    }
}

/// Worker that polls a queue and executes tasks.
pub struct Worker {
    pub queue: Arc<Mutex<dyn TaskQueue + Send>>,
    pub registry: Arc<TaskRegistry>,
    pub running: Arc<std::sync::atomic::AtomicBool>,
}

impl Worker {
    pub fn new(queue: Arc<Mutex<dyn TaskQueue + Send>>, registry: Arc<TaskRegistry>) -> Self {
        Self {
            queue,
            registry,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Process a single task from the queue. Returns true if a task was processed.
    pub fn process_one(&self) -> bool {
        let task = {
            let mut q = self.queue.lock().unwrap();
            q.dequeue()
        };

        if let Some(mut task) = task {
            let result = if let Some(def) = self.registry.get(&task.name) {
                def.execute(task.args.clone())
            } else {
                TaskResult::failure(format!("Unknown task: {}", task.name))
            };

            let mut q = self.queue.lock().unwrap();
            if let Some(stored) = q.get_task(&task.id) {
                task = stored;
            }

            task.result = Some(result.clone());
            task.status = if result.success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };

            // Update the stored task
            let _ = q.cancel(&task.id);
            q.enqueue(task);
            true
        } else {
            false
        }
    }

    /// Run the worker loop (blocking).
    pub fn run(&self, count: Option<usize>) {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        let max = count.unwrap_or(usize::MAX);
        let mut processed = 0;
        while processed < max && self.running.load(std::sync::atomic::Ordering::SeqCst) {
            if self.process_one() {
                processed += 1;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Macro to register a task function in the global registry.
#[macro_export]
macro_rules! register_task {
    ($name:expr, $func:expr) => {
        {
            let registry = $crate::global_registry();
            registry.register($crate::TaskDef::new(
                $name,
                std::sync::Arc::new($func),
                "",
            ));
        }
    };
    ($name:expr, $func:expr, $desc:expr) => {
        {
            let registry = $crate::global_registry();
            registry.register($crate::TaskDef::new(
                $name,
                std::sync::Arc::new($func),
                $desc,
            ));
        }
    };
}

/// Send a task to the default database-backed queue.
pub fn enqueue(name: impl Into<String>, args: HashMap<String, String>) -> TaskHandle {
    let task = Task::new(name, args);
    let id = task.id.clone();

    // For now, use a global memory queue as the default
    static QUEUE: std::sync::LazyLock<Arc<Mutex<MemoryTaskQueue>>> =
        std::sync::LazyLock::new(|| Arc::new(Mutex::new(MemoryTaskQueue::new())));

    {
        let mut q = QUEUE.lock().unwrap();
        q.enqueue(task);
    }

    TaskHandle {
        id,
        queue: QUEUE.clone() as Arc<Mutex<dyn TaskQueue + Send>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TaskStatus tests ─────────────────────────────────────────────

    #[test]
    fn test_task_status_as_str() {
        assert_eq!(TaskStatus::Queued.as_str(), "queued");
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
        assert_eq!(TaskStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn test_task_status_from_str() {
        assert_eq!(TaskStatus::from_str("queued"), TaskStatus::Queued);
        assert_eq!(TaskStatus::from_str("running"), TaskStatus::Running);
        assert_eq!(TaskStatus::from_str("unknown"), TaskStatus::Queued);
    }

    // ── TaskResult tests ─────────────────────────────────────────────

    #[test]
    fn test_task_result_success() {
        let r = TaskResult::success("done!");
        assert!(r.success);
        assert_eq!(r.output, "done!");
        assert!(r.error.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let r = TaskResult::failure("boom");
        assert!(!r.success);
        assert_eq!(r.error.unwrap(), "boom");
    }

    // ── Task tests ───────────────────────────────────────────────────

    #[test]
    fn test_task_new() {
        let mut args = HashMap::new();
        args.insert("url".into(), "https://example.com".into());
        let task = Task::new("fetch_url", args.clone());
        assert_eq!(task.name, "fetch_url");
        assert_eq!(task.args.get("url").unwrap(), "https://example.com");
        assert_eq!(task.status, TaskStatus::Queued);
        assert!(uuid::Uuid::parse_str(&task.id).is_ok());
        assert!(task.scheduled_at.is_none());
    }

    #[test]
    fn test_task_scheduled() {
        let task = Task::scheduled("send_reminder", HashMap::new(), "2026-07-01T00:00:00Z");
        assert_eq!(task.scheduled_at.unwrap(), "2026-07-01T00:00:00Z");
    }

    // ── TaskQueue tests ──────────────────────────────────────────────

    #[test]
    fn test_memory_queue_enqueue_dequeue() {
        let mut queue = MemoryTaskQueue::new();
        assert!(queue.is_empty());

        let task = Task::new("test", HashMap::new());
        let id = queue.enqueue(task);
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let dequeued = queue.dequeue();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id, id);
        // After dequeue, the task is "running" but still stored
        assert_eq!(queue.len(), 0); // dequeued from queue
    }

    #[test]
    fn test_memory_queue_skip_non_queued() {
        let mut queue = MemoryTaskQueue::new();
        let mut task = Task::new("test", HashMap::new());
        task.status = TaskStatus::Running;
        queue.enqueue(task);

        // Should not return the running task
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_memory_queue_cancel() {
        let mut queue = MemoryTaskQueue::new();
        let id = queue.enqueue(Task::new("test", HashMap::new()));
        assert!(queue.cancel(&id));
        let task = queue.get_task(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_memory_queue_cancel_running_fails() {
        let mut queue = MemoryTaskQueue::new();
        let mut task = Task::new("test", HashMap::new());
        task.status = TaskStatus::Running;
        let id = queue.enqueue(task);
        assert!(!queue.cancel(&id));
    }

    #[test]
    fn test_memory_queue_get_task() {
        let mut queue = MemoryTaskQueue::new();
        let id = queue.enqueue(Task::new("test", HashMap::new()));
        let task = queue.get_task(&id);
        assert!(task.is_some());
        assert!(queue.get_task("nonexistent").is_none());
    }

    #[test]
    fn test_memory_queue_completed_and_failed() {
        let mut queue = MemoryTaskQueue::new();

        let mut t1 = Task::new("success", HashMap::new());
        t1.status = TaskStatus::Completed;
        t1.result = Some(TaskResult::success("ok"));
        queue.enqueue(t1);

        let mut t2 = Task::new("fail", HashMap::new());
        t2.status = TaskStatus::Failed;
        t2.result = Some(TaskResult::failure("err"));
        queue.enqueue(t2);

        assert_eq!(queue.completed().len(), 1);
        assert_eq!(queue.failed().len(), 1);
        assert_eq!(queue.all().len(), 2);
    }

    // ── TaskRegistry tests ───────────────────────────────────────────

    #[test]
    fn test_task_registry_register_and_get() {
        let registry = TaskRegistry::new();
        let def = TaskDef::new("greet", Arc::new(|args| {
            let name = args.get("name").cloned().unwrap_or("world".into());
            TaskResult::success(format!("Hello, {}!", name))
        }), "A friendly greeting");

        registry.register(def);
        assert!(registry.contains("greet"));

        let fetched = registry.get("greet");
        assert!(fetched.is_some());

        let result = fetched.unwrap().execute({
            let mut m = HashMap::new();
            m.insert("name".into(), "Rusho".into());
            m
        });
        assert!(result.success);
        assert_eq!(result.output, "Hello, Rusho!");
    }

    #[test]
    fn test_task_registry_all() {
        let registry = TaskRegistry::new();
        registry.register(TaskDef::new("a", Arc::new(|_| TaskResult::success("")), ""));
        registry.register(TaskDef::new("b", Arc::new(|_| TaskResult::success("")), ""));
        assert_eq!(registry.all().len(), 2);
    }

    #[test]
    fn test_task_registry_missing() {
        let registry = TaskRegistry::new();
        assert!(!registry.contains("nonexistent"));
        assert!(registry.get("nonexistent").is_none());
    }

    // ── TaskHandle tests ─────────────────────────────────────────────

    #[test]
    fn test_task_handle() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let id = {
            let mut q = queue.lock().unwrap();
            q.enqueue(Task::new("test", HashMap::new()))
        };
        let handle = TaskHandle::new(id.clone(), queue.clone());

        assert_eq!(handle.status(), TaskStatus::Queued);
        assert!(handle.cancel());
        assert_eq!(handle.status(), TaskStatus::Cancelled);
    }

    #[test]
    fn test_task_handle_cancel_running() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let id = {
            let mut q = queue.lock().unwrap();
            let mut task = Task::new("test", HashMap::new());
            task.status = TaskStatus::Running;
            q.enqueue(task)
        };
        let handle = TaskHandle::new(id, queue);
        assert!(!handle.cancel());
    }

    // ── Worker tests ─────────────────────────────────────────────────

    #[test]
    fn test_worker_process_one() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let registry = Arc::new(TaskRegistry::new());

        registry.register(TaskDef::new("echo", Arc::new(|args| {
            let msg = args.get("msg").cloned().unwrap_or_default();
            TaskResult::success(msg)
        }), "Echoes a message"));

        {
            let mut q = queue.lock().unwrap();
            let mut args = HashMap::new();
            args.insert("msg".into(), "hello".into());
            q.enqueue(Task::new("echo", args));
        }

        let worker = Worker::new(queue.clone(), registry);

        // Should process the one queued task
        assert!(worker.process_one());

        // Queue should have the completed task now
        let tasks = {
            let q = queue.lock().unwrap();
            q.all()
        };
        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.result.as_ref().unwrap().success);
        assert!(task.result.as_ref().unwrap().output.contains("hello"));
    }

    #[test]
    fn test_worker_process_unknown_task() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let registry = Arc::new(TaskRegistry::new());

        {
            let mut q = queue.lock().unwrap();
            q.enqueue(Task::new("undefined_task", HashMap::new()));
        }

        let worker = Worker::new(queue.clone(), registry);
        assert!(worker.process_one());

        let tasks = {
            let q = queue.lock().unwrap();
            q.failed()
        };
        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.status, TaskStatus::Failed);
        assert!(task.result.as_ref().unwrap().error.as_ref().unwrap().contains("undefined_task"));
    }

    #[test]
    fn test_worker_empty_queue() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let registry = Arc::new(TaskRegistry::new());
        let worker = Worker::new(queue, registry);
        assert!(!worker.process_one());
    }

    #[test]
    fn test_worker_run_and_stop() {
        let queue: Arc<Mutex<dyn TaskQueue + Send>> = Arc::new(Mutex::new(MemoryTaskQueue::new()));
        let registry = Arc::new(TaskRegistry::new());

        registry.register(TaskDef::new("quick", Arc::new(|_| TaskResult::success("done")), ""));

        {
            let mut q = queue.lock().unwrap();
            q.enqueue(Task::new("quick", HashMap::new()));
        }

        let worker = Worker::new(queue.clone(), registry);
        worker.run(Some(1)); // Process exactly 1
        // After run, the task should be completed
        let tasks = {
            let q = queue.lock().unwrap();
            q.completed()
        };
        assert_eq!(tasks.len(), 1);
    }

    // ─── enqueue function test ──────────────────────────────────────

    #[test]
    fn test_enqueue_function() {
        let handle = enqueue("test_task", HashMap::new());
        assert!(uuid::Uuid::parse_str(&handle.id).is_ok());
    }

    // ─── register_task! macro test ──────────────────────────────────

    #[test]
    fn test_register_task_macro() {
        register_task!("macro_test", |args: HashMap<String, String>| {
            TaskResult::success(format!("Macro says {}", args.get("x").map(|s| s.as_str()).unwrap_or("?")))
        }, "Macro test task");

        let registry = global_registry();
        assert!(registry.contains("macro_test"));

        let def = registry.get("macro_test").unwrap();
        let result = def.execute({
            let mut m = HashMap::new();
            m.insert("x".into(), "hi".into());
            m
        });
        assert!(result.success);
        assert_eq!(result.output, "Macro says hi");
    }
}
