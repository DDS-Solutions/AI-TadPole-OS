//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Task Backlog**: Orchestrates the async task queue and dependency
//! management for complex missions. Implements the **Backlog Pattern**,
//! enabling agents to decompose high-level objectives into a DAG (Directed
//! Acyclic Graph) of sub-tasks. Provides **Dynamic Priority Resolution**
//! to ensure blocked tasks are only scheduled once their dependencies
//! enter the `Completed` state.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Deadlocks in the task DAG (circular dependencies),
//!   orphaned tasks with missing parent references, or invalid status
//!   transitions (e.g., Blocked -> InProgress without dependency resolution).
//! - **Trace Scope**: `server-rs::agent::backlog`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTask {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub assigned_to: Option<String>, // Agent ID
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
pub struct MissionBacklog {
    pub mission_id: String,
    pub tasks: HashMap<String, MissionTask>,
}

#[allow(dead_code)]
impl MissionBacklog {
    pub fn new(mission_id: &str) -> Self {
        Self {
            mission_id: mission_id.to_string(),
            tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, description: &str, dependencies: Vec<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let task = MissionTask {
            id: id.clone(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            dependencies,
            result: None,
            assigned_to: None,
            created_at: Utc::now(),
        };
        self.tasks.insert(id.clone(), task);
        id
    }

    pub fn update_status(&mut self, task_id: &str, status: TaskStatus, result: Option<String>) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            task.result = result;
        }
    }

    pub fn get_next_available(&self) -> Vec<&MissionTask> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Pending)
            .filter(|t| {
                t.dependencies.iter().all(|dep_id| {
                    self.tasks
                        .get(dep_id)
                        .map(|dep| dep.status == TaskStatus::Completed)
                        .unwrap_or(false)
                })
            })
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.tasks
            .values()
            .all(|t| t.status == TaskStatus::Completed)
    }

    pub fn progress_report(&self) -> String {
        let mut report = format!("Mission Backlog: {}\n", self.mission_id);
        for task in self.tasks.values() {
            let status_icon = match task.status {
                TaskStatus::Pending => "⏳",
                TaskStatus::InProgress => "🏃",
                TaskStatus::Completed => "✅",
                TaskStatus::Failed => "❌",
                TaskStatus::Blocked => "🛑",
            };
            report.push_str(&format!(
                "{} [{}] {}\n",
                status_icon, task.id, task.description
            ));
        }
        report
    }
}
