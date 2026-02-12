//! Temporal Analysis Engine
//!
//! This module tracks code evolution over time through checkpoints, providing
//! insights into how code changes, patterns evolve, and architecture shifts.

use super::types::*;
use crate::error::{CheckpointError, Result};
use crate::types::CheckpointId;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Temporal analysis engine for tracking code evolution
pub struct TemporalAnalyzer {
    checkpoint_timeline: Vec<TemporalCheckpoint>,
    entity_history: HashMap<String, Vec<EntityEvolution>>,
    pattern_history: HashMap<String, Vec<PatternEvolution>>,
    architecture_history: Vec<ArchitectureSnapshot>,
}

/// Checkpoint with temporal metadata
#[derive(Debug, Clone)]
pub struct TemporalCheckpoint {
    pub checkpoint_id: CheckpointId,
    pub timestamp: DateTime<Utc>,
    pub entities: HashMap<String, EntityDefinition>,
    pub patterns: Vec<DetectedPattern>,
    pub architectural_state: ArchitecturalState,
}

/// Evolution of a single code entity over time
#[derive(Debug, Clone)]
pub struct EntityEvolution {
    pub entity_id: String,
    pub checkpoint_id: CheckpointId,
    pub timestamp: DateTime<Utc>,
    pub change_type: EntityChangeType,
    pub entity_state: EntityDefinition,
    pub diff: Option<EntityDiff>,
}

/// Types of entity changes
#[derive(Debug, Clone, PartialEq)]
pub enum EntityChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
    Refactored,
}

/// Difference between entity states
#[derive(Debug, Clone)]
pub struct EntityDiff {
    pub signature_changed: bool,
    pub body_changed: bool,
    pub visibility_changed: bool,
    pub documentation_changed: bool,
    pub dependencies_changed: Vec<String>,
}

/// Evolution of design patterns
#[derive(Debug, Clone)]
pub struct PatternEvolution {
    pub pattern_name: String,
    pub checkpoint_id: CheckpointId,
    pub timestamp: DateTime<Utc>,
    pub evolution_type: PatternEvolutionType,
    pub confidence: f64,
    pub affected_entities: Vec<String>,
}

/// Types of pattern evolution
#[derive(Debug, Clone, PartialEq)]
pub enum PatternEvolutionType {
    Introduced,
    Strengthened,
    Weakened,
    Removed,
    Refactored,
}

/// Snapshot of architectural state
#[derive(Debug, Clone)]
pub struct ArchitectureSnapshot {
    pub checkpoint_id: CheckpointId,
    pub timestamp: DateTime<Utc>,
    pub layers: Vec<String>,
    pub components: Vec<String>,
    pub dependencies: Vec<(String, String)>,
    pub complexity_metrics: ComplexityMetrics,
}

/// Architectural state at a point in time
#[derive(Debug, Clone)]
pub struct ArchitecturalState {
    pub layers: Vec<String>,
    pub components: Vec<String>,
    pub module_count: usize,
    pub average_coupling: f64,
    pub average_cohesion: f64,
}

/// Complexity metrics
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
}

/// Trend analysis result
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub metric_name: String,
    pub trend_direction: TrendDirection,
    pub rate_of_change: f64,
    pub confidence: f64,
    pub time_window: (DateTime<Utc>, DateTime<Utc>),
}

/// Direction of a trend
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

impl TemporalAnalyzer {
    /// Create a new temporal analyzer
    pub fn new() -> Self {
        Self {
            checkpoint_timeline: Vec::new(),
            entity_history: HashMap::new(),
            pattern_history: HashMap::new(),
            architecture_history: Vec::new(),
        }
    }

    /// Add a checkpoint to the temporal timeline
    pub fn add_checkpoint(&mut self, checkpoint: TemporalCheckpoint) -> Result<()> {
        // Insert in chronological order
        let pos = self
            .checkpoint_timeline
            .binary_search_by_key(&checkpoint.timestamp, |c| c.timestamp)
            .unwrap_or_else(|e| e);

        self.checkpoint_timeline.insert(pos, checkpoint.clone());

        // Update entity history
        self.update_entity_history(&checkpoint)?;

        // Update pattern history
        self.update_pattern_history(&checkpoint)?;

        // Update architecture history
        self.update_architecture_history(&checkpoint)?;

        Ok(())
    }

    /// Update entity history with new checkpoint
    fn update_entity_history(&mut self, checkpoint: &TemporalCheckpoint) -> Result<()> {
        for (entity_id, entity) in &checkpoint.entities {
            let evolution = EntityEvolution {
                entity_id: entity_id.clone(),
                checkpoint_id: checkpoint.checkpoint_id,
                timestamp: checkpoint.timestamp,
                change_type: self.determine_entity_change_type(entity_id, entity),
                entity_state: entity.clone(),
                diff: self.calculate_entity_diff(entity_id, entity),
            };

            self.entity_history
                .entry(entity_id.clone())
                .or_insert_with(Vec::new)
                .push(evolution);
        }

        Ok(())
    }

    /// Determine the type of change for an entity
    fn determine_entity_change_type(
        &self,
        entity_id: &str,
        _entity: &EntityDefinition,
    ) -> EntityChangeType {
        if let Some(history) = self.entity_history.get(entity_id) {
            if history.is_empty() {
                EntityChangeType::Created
            } else {
                // Check if entity was recently deleted and recreated
                if let Some(last) = history.last() {
                    if last.change_type == EntityChangeType::Deleted {
                        EntityChangeType::Created
                    } else {
                        EntityChangeType::Modified
                    }
                } else {
                    EntityChangeType::Modified
                }
            }
        } else {
            EntityChangeType::Created
        }
    }

    /// Calculate difference between entity states
    fn calculate_entity_diff(
        &self,
        entity_id: &str,
        current_entity: &EntityDefinition,
    ) -> Option<EntityDiff> {
        if let Some(history) = self.entity_history.get(entity_id) {
            if let Some(last_evolution) = history.last() {
                let previous = &last_evolution.entity_state;

                return Some(EntityDiff {
                    signature_changed: previous.name() != current_entity.name()
                        || previous.entity_type_name() != current_entity.entity_type_name(),
                    body_changed: true, // Would need content comparison
                    visibility_changed: previous.visibility() != current_entity.visibility(),
                    documentation_changed: previous.documentation()
                        != current_entity.documentation(),
                    dependencies_changed: Vec::new(), // Would need dependency comparison
                });
            }
        }

        None
    }

    /// Update pattern history
    fn update_pattern_history(&mut self, checkpoint: &TemporalCheckpoint) -> Result<()> {
        for pattern in &checkpoint.patterns {
            let evolution_type = self.determine_pattern_evolution(&pattern.name, checkpoint);

            let evolution = PatternEvolution {
                pattern_name: pattern.name.clone(),
                checkpoint_id: checkpoint.checkpoint_id,
                timestamp: checkpoint.timestamp,
                evolution_type,
                confidence: pattern.confidence,
                affected_entities: pattern.locations.clone(),
            };

            self.pattern_history
                .entry(pattern.name.clone())
                .or_insert_with(Vec::new)
                .push(evolution);
        }

        Ok(())
    }

    /// Determine pattern evolution type
    fn determine_pattern_evolution(
        &self,
        pattern_name: &str,
        checkpoint: &TemporalCheckpoint,
    ) -> PatternEvolutionType {
        if let Some(history) = self.pattern_history.get(pattern_name) {
            if let Some(last) = history.last() {
                // Get current pattern
                let current_pattern = checkpoint.patterns.iter().find(|p| p.name == pattern_name);

                if let Some(current) = current_pattern {
                    if current.confidence > last.confidence {
                        PatternEvolutionType::Strengthened
                    } else if current.confidence < last.confidence {
                        PatternEvolutionType::Weakened
                    } else {
                        PatternEvolutionType::Refactored
                    }
                } else {
                    PatternEvolutionType::Removed
                }
            } else {
                PatternEvolutionType::Introduced
            }
        } else {
            PatternEvolutionType::Introduced
        }
    }

    /// Update architecture history
    fn update_architecture_history(&mut self, checkpoint: &TemporalCheckpoint) -> Result<()> {
        let snapshot = ArchitectureSnapshot {
            checkpoint_id: checkpoint.checkpoint_id,
            timestamp: checkpoint.timestamp,
            layers: checkpoint.architectural_state.layers.clone(),
            components: checkpoint.architectural_state.components.clone(),
            dependencies: Vec::new(), // Would extract from checkpoint
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 0.0,
                cognitive_complexity: 0.0,
                maintainability_index: 0.0,
                technical_debt_ratio: 0.0,
            },
        };

        self.architecture_history.push(snapshot);

        Ok(())
    }

    /// Get entity history for a specific entity
    pub fn get_entity_history(&self, entity_id: &str) -> Option<&Vec<EntityEvolution>> {
        self.entity_history.get(entity_id)
    }

    /// Get pattern evolution history
    pub fn get_pattern_history(&self, pattern_name: &str) -> Option<&Vec<PatternEvolution>> {
        self.pattern_history.get(pattern_name)
    }

    /// Get all entities that changed between two checkpoints
    pub fn get_changes_between(
        &self,
        from_checkpoint: CheckpointId,
        to_checkpoint: CheckpointId,
    ) -> Result<Vec<EntityEvolution>> {
        let from_pos = self
            .checkpoint_timeline
            .iter()
            .position(|c| c.checkpoint_id == from_checkpoint)
            .ok_or_else(|| CheckpointError::validation("From checkpoint not found".to_string()))?;

        let to_pos = self
            .checkpoint_timeline
            .iter()
            .position(|c| c.checkpoint_id == to_checkpoint)
            .ok_or_else(|| CheckpointError::validation("To checkpoint not found".to_string()))?;

        let mut changes = Vec::new();

        for (_, history) in &self.entity_history {
            for evolution in history {
                // Find checkpoint index
                if let Some(checkpoint_pos) = self
                    .checkpoint_timeline
                    .iter()
                    .position(|c| c.checkpoint_id == evolution.checkpoint_id)
                {
                    if checkpoint_pos > from_pos && checkpoint_pos <= to_pos {
                        changes.push(evolution.clone());
                    }
                }
            }
        }

        Ok(changes)
    }

    /// Analyze trends over time
    pub fn analyze_trends(&self, window_size: usize) -> Vec<TrendAnalysis> {
        let mut trends = Vec::new();

        if self.architecture_history.len() < window_size {
            return trends;
        }

        // Analyze complexity trends
        let complexity_trend = self.analyze_complexity_trend(window_size);
        if let Some(trend) = complexity_trend {
            trends.push(trend);
        }

        // Analyze coupling trends
        let coupling_trend = self.analyze_coupling_trend(window_size);
        if let Some(trend) = coupling_trend {
            trends.push(trend);
        }

        // Analyze pattern adoption trends
        let pattern_trends = self.analyze_pattern_trends(window_size);
        trends.extend(pattern_trends);

        trends
    }

    /// Analyze complexity trend
    fn analyze_complexity_trend(&self, window_size: usize) -> Option<TrendAnalysis> {
        let recent_snapshots: Vec<_> = self
            .architecture_history
            .iter()
            .rev()
            .take(window_size)
            .collect();

        if recent_snapshots.len() < 2 {
            return None;
        }

        let complexities: Vec<_> = recent_snapshots
            .iter()
            .map(|s| s.complexity_metrics.cyclomatic_complexity)
            .collect();

        let trend_direction = self.determine_trend_direction(&complexities);
        let rate_of_change = self.calculate_rate_of_change(&complexities);

        Some(TrendAnalysis {
            metric_name: "Cyclomatic Complexity".to_string(),
            trend_direction,
            rate_of_change,
            confidence: 0.8,
            time_window: (
                recent_snapshots.last().unwrap().timestamp,
                recent_snapshots.first().unwrap().timestamp,
            ),
        })
    }

    /// Analyze coupling trend
    fn analyze_coupling_trend(&self, window_size: usize) -> Option<TrendAnalysis> {
        let recent_checkpoints: Vec<_> = self
            .checkpoint_timeline
            .iter()
            .rev()
            .take(window_size)
            .collect();

        if recent_checkpoints.len() < 2 {
            return None;
        }

        let coupling_values: Vec<_> = recent_checkpoints
            .iter()
            .map(|c| c.architectural_state.average_coupling)
            .collect();

        let trend_direction = self.determine_trend_direction(&coupling_values);
        let rate_of_change = self.calculate_rate_of_change(&coupling_values);

        Some(TrendAnalysis {
            metric_name: "Average Coupling".to_string(),
            trend_direction,
            rate_of_change,
            confidence: 0.75,
            time_window: (
                recent_checkpoints.last().unwrap().timestamp,
                recent_checkpoints.first().unwrap().timestamp,
            ),
        })
    }

    /// Analyze pattern trends
    fn analyze_pattern_trends(&self, window_size: usize) -> Vec<TrendAnalysis> {
        let mut trends = Vec::new();

        for (pattern_name, history) in &self.pattern_history {
            if history.len() < window_size {
                continue;
            }

            let recent_history: Vec<_> = history.iter().rev().take(window_size).collect();

            let confidence_values: Vec<_> = recent_history.iter().map(|e| e.confidence).collect();

            let trend_direction = self.determine_trend_direction(&confidence_values);
            let rate_of_change = self.calculate_rate_of_change(&confidence_values);

            trends.push(TrendAnalysis {
                metric_name: format!("Pattern: {}", pattern_name),
                trend_direction,
                rate_of_change,
                confidence: 0.7,
                time_window: (
                    recent_history.last().unwrap().timestamp,
                    recent_history.first().unwrap().timestamp,
                ),
            });
        }

        trends
    }

    /// Determine trend direction from a series of values
    fn determine_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let mut increases = 0;
        let mut decreases = 0;

        for i in 1..values.len() {
            if values[i] > values[i - 1] {
                increases += 1;
            } else if values[i] < values[i - 1] {
                decreases += 1;
            }
        }

        let total_changes = increases + decreases;
        if total_changes == 0 {
            return TrendDirection::Stable;
        }

        let increase_ratio = increases as f64 / total_changes as f64;

        if increase_ratio > 0.7 {
            TrendDirection::Increasing
        } else if increase_ratio < 0.3 {
            TrendDirection::Decreasing
        } else if increases > 0 && decreases > 0 {
            TrendDirection::Volatile
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate rate of change
    fn calculate_rate_of_change(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values.first().unwrap();
        let last = values.last().unwrap();

        if *first == 0.0 {
            return 0.0;
        }

        (last - first) / first
    }

    /// Find entities that changed frequently
    pub fn find_hot_spots(&self, threshold: usize) -> Vec<(String, usize)> {
        let mut change_counts: HashMap<String, usize> = HashMap::new();

        for (entity_id, history) in &self.entity_history {
            change_counts.insert(entity_id.clone(), history.len());
        }

        let mut hot_spots: Vec<_> = change_counts
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .collect();

        hot_spots.sort_by(|a, b| b.1.cmp(&a.1));
        hot_spots
    }

    /// Predict future complexity based on trends
    pub fn predict_complexity(&self, checkpoints_ahead: usize) -> Option<f64> {
        if self.architecture_history.len() < 3 {
            return None;
        }

        let recent_complexities: Vec<_> = self
            .architecture_history
            .iter()
            .rev()
            .take(5)
            .map(|s| s.complexity_metrics.cyclomatic_complexity)
            .collect();

        let rate = self.calculate_rate_of_change(&recent_complexities);
        let current = recent_complexities.first()?;

        Some(current * (1.0 + rate * checkpoints_ahead as f64))
    }

    /// Get checkpoint timeline
    pub fn get_timeline(&self) -> &[TemporalCheckpoint] {
        &self.checkpoint_timeline
    }

    /// Get architecture history
    pub fn get_architecture_history(&self) -> &[ArchitectureSnapshot] {
        &self.architecture_history
    }
}

impl Default for TemporalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_analyzer_creation() {
        let analyzer = TemporalAnalyzer::new();
        assert_eq!(analyzer.checkpoint_timeline.len(), 0);
    }

    #[test]
    fn test_determine_trend_direction() {
        let analyzer = TemporalAnalyzer::new();

        let increasing = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            analyzer.determine_trend_direction(&increasing),
            TrendDirection::Increasing
        );

        let decreasing = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert_eq!(
            analyzer.determine_trend_direction(&decreasing),
            TrendDirection::Decreasing
        );

        let stable = vec![3.0, 3.0, 3.0, 3.0, 3.0];
        assert_eq!(
            analyzer.determine_trend_direction(&stable),
            TrendDirection::Stable
        );
    }
}
