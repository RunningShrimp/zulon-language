// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Learning curve measurement framework
//!
//! Provides tools for measuring developer productivity and learning curves

use std::collections::HashMap;
use std::io::Write;
use std::time::{Duration, Instant};

/// Task difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Beginner level task
    Beginner,
    /// Intermediate level task
    Intermediate,
    /// Advanced level task
    Advanced,
    /// Expert level task
    Expert,
}

/// Programming task for learning curve measurement
#[derive(Debug, Clone)]
pub struct ProgrammingTask {
    /// Task identifier
    pub id: String,
    /// Task name
    pub name: String,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Expected time to complete (in minutes)
    pub expected_time_minutes: u32,
    /// Task description
    pub description: String,
}

/// Learning curve measurement result
#[derive(Debug, Clone)]
pub struct LearningCurveResult {
    /// Participant ID
    pub participant_id: String,
    /// Task ID
    pub task_id: String,
    /// Time taken to complete (in seconds)
    pub time_taken_seconds: u64,
    /// Number of attempts
    pub attempts: u32,
    /// Success indicator
    pub success: bool,
}

/// Learning curve statistics
#[derive(Debug, Clone)]
pub struct LearningCurveStats {
    /// Total tasks attempted
    pub total_tasks: usize,
    /// Total tasks completed
    pub completed_tasks: usize,
    /// Average time per task (in seconds)
    pub avg_time_per_task: f64,
    /// Success rate
    pub success_rate: f64,
}

/// Learning curve measurement framework
pub struct LearningCurveFramework {
    tasks: HashMap<String, ProgrammingTask>,
    results: Vec<LearningCurveResult>,
}

impl LearningCurveFramework {
    pub fn new() -> Self {
        LearningCurveFramework {
            tasks: HashMap::new(),
            results: Vec::new(),
        }
    }

    /// Add a programming task
    pub fn add_task(&mut self, task: ProgrammingTask) {
        self.tasks.insert(task.id.clone(), task);
    }

    /// Record learning curve measurement
    pub fn record_result(&mut self, result: LearningCurveResult) {
        self.results.push(result);
    }

    /// Calculate learning curve statistics
    pub fn calculate_stats(&self, participant_id: &str) -> Option<LearningCurveStats> {
        let participant_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.participant_id == participant_id)
            .collect();

        if participant_results.is_empty() {
            return None;
        }

        let total_tasks = participant_results.len();
        let completed_tasks = participant_results.iter().filter(|r| r.success).count();
        let success_rate = completed_tasks as f64 / total_tasks as f64;

        let total_time: u64 = participant_results
            .iter()
            .map(|r| r.time_taken_seconds)
            .sum();
        let avg_time = total_time as f64 / total_tasks as f64;

        Some(LearningCurveStats {
            total_tasks,
            completed_tasks,
            avg_time_per_task: avg_time,
            success_rate,
        })
    }

    /// Calculate time to proficiency (learning rate)
    pub fn calculate_time_to_proficiency(
        &self,
        participant_id: &str,
        difficulty: Difficulty,
    ) -> f64 {
        let stats = match self.calculate_stats(participant_id) {
            Some(s) => s,
            None => return 0.0,
        };

        let expected_time = self
            .tasks
            .iter()
            .find(|t| t.difficulty == difficulty)
            .map(|t| t.expected_time_minutes as f64 * 60.0)
            .unwrap_or(300.0);

        if stats.avg_time_per_task > 0.0 {
            expected_time / stats.avg_time_per_task
        } else {
            0.0
        }
    }

    /// Generate learning curve report
    pub fn generate_report<W: Write>(
        &self,
        writer: &mut W,
        participant_id: &str,
    ) -> std::io::Result<()> {
        writeln!(writer, "\n=== Learning Curve Report ===")?;
        writeln!(writer, "Participant: {}", participant_id)?;

        if let Some(stats) = self.calculate_stats(participant_id) {
            writeln!(writer, "Total Tasks: {}", stats.total_tasks)?;
            writeln!(writer, "Completed: {}", stats.completed_tasks)?;
            writeln!(writer, "Success Rate: {:.2}%", stats.success_rate)?;
            writeln!(writer, "Avg Time per Task: {:.2}s", stats.avg_time_per_task)?;
        }

        // Calculate proficiency by difficulty
        writeln!(writer, "\n=== Proficiency by Difficulty ===")?;
        for difficulty in &[
            Difficulty::Beginner,
            Difficulty::Intermediate,
            Difficulty::Advanced,
            Difficulty::Expert,
        ] {
            let proficiency = self.calculate_time_to_proficiency(participant_id, difficulty);
            writeln!(writer, "{:?}: {:.2}", difficulty, proficiency)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_record_task() {
        let mut framework = LearningCurveFramework::new();

        let task = ProgrammingTask {
            id: "task1".to_string(),
            name: "Hello World".to_string(),
            difficulty: Difficulty::Beginner,
            expected_time_minutes: 5,
            description: "Write a simple program".to_string(),
        };

        framework.add_task(task);

        let result = LearningCurveResult {
            participant_id: "user1".to_string(),
            task_id: "task1".to_string(),
            time_taken_seconds: 120,
            attempts: 1,
            success: true,
        };

        framework.record_result(result);

        let stats = framework.calculate_stats("user1").unwrap();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.completed_tasks, 1);
    }
}
