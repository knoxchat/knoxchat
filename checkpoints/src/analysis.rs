//! Phase 8.3: AI-Powered Checkpoint Analysis
//!
//! Provides intelligent analysis of checkpoint changes including:
//! - Auto-generated descriptions from code diffs
//! - Risk assessment for changes
//! - Impact analysis across features and layers
//! - Checkpoint grouping suggestions

use crate::db::CheckpointDatabase;
use crate::error::Result;
use crate::types::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// AI-powered checkpoint analyzer
pub struct CheckpointAnalyzer {
    database: Arc<CheckpointDatabase>,
}

impl CheckpointAnalyzer {
    /// Create a new checkpoint analyzer
    pub fn new(database: Arc<CheckpointDatabase>) -> Self {
        Self { database }
    }

    /// Analyze a checkpoint and return comprehensive analysis
    pub fn analyze_checkpoint(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointAnalysis> {
        // Check cache first
        if let Some(cached) = self.database.get_analysis(checkpoint_id)? {
            return Ok(cached);
        }

        let checkpoint = self
            .database
            .get_checkpoint(checkpoint_id)?
            .ok_or_else(|| {
                crate::error::CheckpointError::checkpoint_not_found(checkpoint_id.to_string())
            })?;

        let analysis = CheckpointAnalysis {
            generated_description: self.generate_description(&checkpoint),
            risk_assessment: self.assess_risk(&checkpoint),
            impact_analysis: self.analyze_impact(&checkpoint),
            grouping_suggestion: self.suggest_grouping(checkpoint_id)?,
        };

        // Cache the result
        let _ = self.database.store_analysis(checkpoint_id, &analysis);

        Ok(analysis)
    }

    /// Auto-generate a human-readable description from code diff
    fn generate_description(&self, checkpoint: &Checkpoint) -> String {
        let changes = &checkpoint.file_changes;
        if changes.is_empty() {
            return "Empty checkpoint - no file changes".to_string();
        }

        let created: Vec<_> = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Created))
            .collect();
        let modified: Vec<_> = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Modified))
            .collect();
        let deleted: Vec<_> = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Deleted))
            .collect();
        let renamed: Vec<_> = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Renamed { .. }))
            .collect();

        // Detect feature areas affected
        let feature_areas = self.detect_feature_areas(changes);

        // Build description
        let mut parts = Vec::new();

        // Summarize by change type
        if !created.is_empty() {
            if created.len() <= 3 {
                let names: Vec<_> = created
                    .iter()
                    .map(|c| file_name(&c.path))
                    .collect();
                parts.push(format!("Added {}", names.join(", ")));
            } else {
                parts.push(format!("Added {} files", created.len()));
            }
        }

        if !modified.is_empty() {
            if modified.len() <= 3 {
                let names: Vec<_> = modified
                    .iter()
                    .map(|c| file_name(&c.path))
                    .collect();
                parts.push(format!("Modified {}", names.join(", ")));
            } else {
                parts.push(format!("Modified {} files", modified.len()));
            }
        }

        if !deleted.is_empty() {
            if deleted.len() <= 3 {
                let names: Vec<_> = deleted
                    .iter()
                    .map(|c| file_name(&c.path))
                    .collect();
                parts.push(format!("Deleted {}", names.join(", ")));
            } else {
                parts.push(format!("Deleted {} files", deleted.len()));
            }
        }

        if !renamed.is_empty() {
            parts.push(format!("Renamed {} files", renamed.len()));
        }

        // Add feature area context
        if !feature_areas.is_empty() {
            let areas: Vec<_> = feature_areas.into_iter().take(3).collect();
            parts.push(format!("in {}", areas.join(", ")));
        }

        if parts.is_empty() {
            "Minor changes".to_string()
        } else {
            parts.join("; ")
        }
    }

    /// Assess risk level of checkpoint changes
    fn assess_risk(&self, checkpoint: &Checkpoint) -> RiskAssessment {
        let mut factors = Vec::new();
        let mut total_weight = 0.0;

        let changes = &checkpoint.file_changes;

        // Factor 1: Config file changes
        let config_changes: Vec<_> = changes
            .iter()
            .filter(|c| is_config_file(&c.path))
            .collect();
        if !config_changes.is_empty() {
            let weight = 0.3 * (config_changes.len() as f64 / changes.len().max(1) as f64);
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::ConfigChange,
                description: format!(
                    "{} configuration file(s) modified",
                    config_changes.len()
                ),
                affected_files: config_changes.iter().map(|c| c.path.clone()).collect(),
                weight,
            });
        }

        // Factor 2: Security-sensitive files
        let security_changes: Vec<_> = changes
            .iter()
            .filter(|c| is_security_sensitive(&c.path))
            .collect();
        if !security_changes.is_empty() {
            let weight = 0.5;
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::SecuritySensitive,
                description: format!(
                    "{} security-sensitive file(s) modified",
                    security_changes.len()
                ),
                affected_files: security_changes.iter().map(|c| c.path.clone()).collect(),
                weight,
            });
        }

        // Factor 3: Mass deletions
        let deletion_count = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Deleted))
            .count();
        if deletion_count > 5 {
            let weight = 0.4 * (deletion_count as f64 / 20.0).min(1.0);
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::MassDeletion,
                description: format!("{} files deleted", deletion_count),
                affected_files: changes
                    .iter()
                    .filter(|c| matches!(c.change_type, ChangeType::Deleted))
                    .map(|c| c.path.clone())
                    .collect(),
                weight,
            });
        }

        // Factor 4: Infrastructure changes
        let infra_changes: Vec<_> = changes
            .iter()
            .filter(|c| is_infrastructure_file(&c.path))
            .collect();
        if !infra_changes.is_empty() {
            let weight = 0.3;
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::InfrastructureChange,
                description: format!(
                    "{} build/deployment file(s) modified",
                    infra_changes.len()
                ),
                affected_files: infra_changes.iter().map(|c| c.path.clone()).collect(),
                weight,
            });
        }

        // Factor 5: API/interface changes
        let api_changes: Vec<_> = changes
            .iter()
            .filter(|c| is_api_file(&c.path))
            .collect();
        if !api_changes.is_empty() {
            let weight = 0.35;
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::ApiChange,
                description: format!("{} API/interface file(s) modified", api_changes.len()),
                affected_files: api_changes.iter().map(|c| c.path.clone()).collect(),
                weight,
            });
        }

        // Factor 6: Database changes
        let db_changes: Vec<_> = changes
            .iter()
            .filter(|c| is_database_file(&c.path))
            .collect();
        if !db_changes.is_empty() {
            let weight = 0.45;
            total_weight += weight;
            factors.push(RiskFactor {
                category: RiskCategory::DatabaseChange,
                description: format!("{} database file(s) modified", db_changes.len()),
                affected_files: db_changes.iter().map(|c| c.path.clone()).collect(),
                weight,
            });
        }

        // Factor 7: High coupling (many interconnected files changed)
        if changes.len() > 10 {
            let dirs: std::collections::HashSet<_> = changes
                .iter()
                .filter_map(|c| c.path.parent().map(|p| p.to_path_buf()))
                .collect();
            if dirs.len() >= 5 {
                let weight = 0.25;
                total_weight += weight;
                factors.push(RiskFactor {
                    category: RiskCategory::HighCoupling,
                    description: format!(
                        "Changes span {} directories ({} files)",
                        dirs.len(),
                        changes.len()
                    ),
                    affected_files: changes.iter().map(|c| c.path.clone()).collect(),
                    weight,
                });
            }
        }

        // Calculate overall score (normalize to 0.0-1.0)
        let score = (total_weight / 2.5).min(1.0); // 2.5 is approximate max weight sum

        let level = match score {
            s if s < 0.2 => RiskLevel::Low,
            s if s < 0.5 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(&factors, level);

        RiskAssessment {
            level,
            score,
            factors,
            recommendations,
        }
    }

    /// Generate risk mitigation recommendations
    fn generate_recommendations(
        &self,
        factors: &[RiskFactor],
        level: RiskLevel,
    ) -> Vec<String> {
        let mut recs = Vec::new();

        for factor in factors {
            match factor.category {
                RiskCategory::ConfigChange => {
                    recs.push("Review configuration changes carefully before deploying".to_string());
                }
                RiskCategory::SecuritySensitive => {
                    recs.push("Security review recommended for auth/crypto changes".to_string());
                }
                RiskCategory::MassDeletion => {
                    recs.push("Verify deleted files are not referenced elsewhere".to_string());
                }
                RiskCategory::InfrastructureChange => {
                    recs.push("Test build pipeline after infrastructure changes".to_string());
                }
                RiskCategory::ApiChange => {
                    recs.push("Check API consumers for compatibility".to_string());
                }
                RiskCategory::DatabaseChange => {
                    recs.push("Ensure database migrations are reversible".to_string());
                }
                RiskCategory::HighCoupling => {
                    recs.push("Consider splitting this change into smaller, focused checkpoints".to_string());
                }
                RiskCategory::Untested => {
                    recs.push("Add tests for untested changes".to_string());
                }
            }
        }

        if matches!(level, RiskLevel::High | RiskLevel::Critical) {
            recs.push("Create a backup before applying these changes".to_string());
        }

        recs
    }

    /// Analyze the impact of changes across features and layers
    fn analyze_impact(&self, checkpoint: &Checkpoint) -> ImpactAnalysis {
        let changes = &checkpoint.file_changes;

        // Detect affected features
        let affected_features = self.detect_affected_features(changes);

        // Detect affected layers
        let affected_layers = self.detect_affected_layers(changes);

        // Determine change scope
        let scope = self.determine_scope(changes);

        // Identify transitive impact (files that might be affected downstream)
        let transitive_impact = self.estimate_transitive_impact(changes);

        ImpactAnalysis {
            affected_features,
            affected_layers,
            scope,
            transitive_impact,
        }
    }

    /// Detect feature areas from file paths
    fn detect_feature_areas(&self, changes: &[FileChange]) -> Vec<String> {
        let mut areas = std::collections::HashSet::new();

        for change in changes {
            if let Some(area) = classify_feature_area(&change.path) {
                areas.insert(area);
            }
        }

        areas.into_iter().collect()
    }

    /// Detect affected features with impact levels
    fn detect_affected_features(&self, changes: &[FileChange]) -> Vec<AffectedFeature> {
        let mut feature_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for change in changes {
            let area = classify_feature_area(&change.path)
                .unwrap_or_else(|| "Other".to_string());
            feature_map
                .entry(area)
                .or_default()
                .push(change.path.clone());
        }

        feature_map
            .into_iter()
            .map(|(name, files)| {
                let impact_level = match files.len() {
                    1 => ImpactLevel::Minor,
                    2..=5 => ImpactLevel::Moderate,
                    _ => ImpactLevel::Major,
                };
                AffectedFeature {
                    name,
                    impact_level,
                    changed_files: files,
                }
            })
            .collect()
    }

    /// Detect architectural layers affected
    fn detect_affected_layers(&self, changes: &[FileChange]) -> Vec<String> {
        let mut layers = std::collections::HashSet::new();

        for change in changes {
            if let Some(layer) = classify_layer(&change.path) {
                layers.insert(layer);
            }
        }

        layers.into_iter().collect()
    }

    /// Determine the scope of changes
    fn determine_scope(&self, changes: &[FileChange]) -> ChangeScope {
        if changes.is_empty() {
            return ChangeScope::Function;
        }

        let dirs: std::collections::HashSet<_> = changes
            .iter()
            .filter_map(|c| c.path.parent().map(|p| p.to_path_buf()))
            .collect();

        let top_level_dirs: std::collections::HashSet<_> = changes
            .iter()
            .filter_map(|c| {
                c.path
                    .components()
                    .next()
                    .map(|comp| comp.as_os_str().to_string_lossy().to_string())
            })
            .collect();

        match (changes.len(), dirs.len(), top_level_dirs.len()) {
            (1, _, _) => ChangeScope::File,
            (_, 1, _) => ChangeScope::Module,
            (_, _, 1) => ChangeScope::Module,
            (_, _, 2..=3) => ChangeScope::CrossModule,
            _ => ChangeScope::Architecture,
        }
    }

    /// Estimate files that might be transitively impacted
    fn estimate_transitive_impact(&self, changes: &[FileChange]) -> Vec<PathBuf> {
        let mut impacted = Vec::new();

        for change in changes {
            // If a type definition or interface file changed, mark related files
            let path_str = change.path.to_string_lossy();

            if path_str.contains("types") || path_str.contains("interface") {
                // Files in the same directory are likely impacted
                if let Some(parent) = change.path.parent() {
                    impacted.push(parent.to_path_buf());
                }
            }

            // If an index/export file changed, downstream importers are affected
            if let Some(stem) = change.path.file_stem() {
                let stem_str = stem.to_string_lossy();
                if stem_str == "index" || stem_str == "exports" || stem_str == "mod" {
                    if let Some(parent) = change.path.parent() {
                        impacted.push(parent.to_path_buf());
                    }
                }
            }
        }

        impacted
    }

    /// Suggest grouping for related checkpoints
    fn suggest_grouping(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<Option<GroupingSuggestion>> {
        // Get recent checkpoints to look for patterns
        let checkpoint = match self.database.get_checkpoint(checkpoint_id)? {
            Some(cp) => cp,
            None => return Ok(None),
        };

        let recent = self
            .database
            .list_checkpoints(&checkpoint.session_id, Some(20))?;

        if recent.len() < 3 {
            return Ok(None);
        }

        // Find checkpoints that modify the same set of files
        let current_dirs: std::collections::HashSet<_> = checkpoint
            .file_changes
            .iter()
            .filter_map(|c| c.path.parent().map(|p| p.to_path_buf()))
            .collect();

        let mut related_ids = vec![*checkpoint_id];
        let mut max_overlap = 0.0f64;

        for cp in &recent {
            if cp.id == *checkpoint_id {
                continue;
            }

            let cp_dirs: std::collections::HashSet<_> = cp
                .file_changes
                .iter()
                .filter_map(|c| c.path.parent().map(|p| p.to_path_buf()))
                .collect();

            if current_dirs.is_empty() || cp_dirs.is_empty() {
                continue;
            }

            let intersection = current_dirs.intersection(&cp_dirs).count();
            let union = current_dirs.union(&cp_dirs).count();
            let overlap = intersection as f64 / union as f64;

            if overlap > 0.5 {
                related_ids.push(cp.id);
                max_overlap = max_overlap.max(overlap);
            }
        }

        if related_ids.len() >= 3 {
            let group_name = if let Some(common_dir) = current_dirs.iter().next() {
                format!(
                    "Changes to {}",
                    common_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "project".to_string())
                )
            } else {
                "Related changes".to_string()
            };

            let count = related_ids.len();
            Ok(Some(GroupingSuggestion {
                group_name,
                checkpoint_ids: related_ids,
                rationale: format!(
                    "These {} checkpoints modify overlapping sets of files",
                    count
                ),
                confidence: max_overlap,
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze multiple checkpoints and suggest groupings
    pub fn suggest_checkpoint_groups(
        &self,
        session_id: &SessionId,
        limit: usize,
    ) -> Result<Vec<GroupingSuggestion>> {
        let checkpoints = self.database.list_checkpoints(session_id, Some(limit))?;
        let mut suggestions = Vec::new();
        let mut processed: std::collections::HashSet<CheckpointId> = std::collections::HashSet::new();

        for cp in &checkpoints {
            if processed.contains(&cp.id) {
                continue;
            }

            if let Some(suggestion) = self.suggest_grouping(&cp.id)? {
                for id in &suggestion.checkpoint_ids {
                    processed.insert(*id);
                }
                suggestions.push(suggestion);
            }
        }

        Ok(suggestions)
    }
}

// ========================================
// Helper functions for file classification
// ========================================

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn is_config_file(path: &Path) -> bool {
    let name = file_name(path).to_lowercase();
    name.contains("config")
        || name.contains("settings")
        || name.ends_with(".env")
        || name.ends_with(".toml")
        || name.ends_with(".ini")
        || name.ends_with(".cfg")
        || name == "package.json"
        || name == "tsconfig.json"
        || name == "cargo.toml"
        || name == ".eslintrc"
        || name == ".prettierrc"
}

fn is_security_sensitive(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("auth")
        || path_str.contains("crypto")
        || path_str.contains("security")
        || path_str.contains("password")
        || path_str.contains("secret")
        || path_str.contains("token")
        || path_str.contains("cert")
        || path_str.contains("key")
        || path_str.ends_with(".pem")
        || path_str.ends_with(".key")
}

fn is_infrastructure_file(path: &Path) -> bool {
    let name = file_name(path).to_lowercase();
    let path_str = path.to_string_lossy().to_lowercase();
    name == "dockerfile"
        || name == "docker-compose.yml"
        || name == "docker-compose.yaml"
        || name.ends_with(".dockerfile")
        || name == "jenkinsfile"
        || name == "makefile"
        || name == "build.js"
        || name == "build.rs"
        || name == "webpack.config.js"
        || name == "vite.config.ts"
        || name == "rollup.config.js"
        || path_str.contains(".github/workflows")
        || path_str.contains(".gitlab-ci")
        || path_str.contains("ci/")
        || path_str.contains("deploy/")
}

fn is_api_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("api")
        || path_str.contains("routes")
        || path_str.contains("endpoints")
        || path_str.contains("controller")
        || path_str.contains("handler")
        || path_str.contains("protocol")
        || path_str.contains("schema")
        || path_str.contains("graphql")
        || path_str.ends_with(".proto")
        || path_str.ends_with(".swagger")
        || path_str.ends_with(".openapi")
}

fn is_database_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let name = file_name(path).to_lowercase();
    path_str.contains("migration")
        || path_str.contains("schema")
        || path_str.contains("db/")
        || path_str.contains("database")
        || name.ends_with(".sql")
        || name.contains("prisma")
        || name.contains("knex")
}

fn classify_feature_area(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("test") || path_str.contains("spec") {
        Some("Testing".to_string())
    } else if path_str.contains("gui") || path_str.contains("ui") || path_str.contains("component") {
        Some("UI/Frontend".to_string())
    } else if path_str.contains("api") || path_str.contains("route") || path_str.contains("controller") {
        Some("API".to_string())
    } else if path_str.contains("auth") || path_str.contains("security") {
        Some("Authentication".to_string())
    } else if path_str.contains("db") || path_str.contains("model") || path_str.contains("migration") {
        Some("Database".to_string())
    } else if path_str.contains("config") || path_str.contains("setting") {
        Some("Configuration".to_string())
    } else if path_str.contains("util") || path_str.contains("helper") || path_str.contains("lib") {
        Some("Utilities".to_string())
    } else if path_str.contains("doc") || path_str.contains("readme") {
        Some("Documentation".to_string())
    } else if path_str.contains("build") || path_str.contains("script") || path_str.contains("ci") {
        Some("Build/CI".to_string())
    } else {
        // Use top-level directory as feature area
        path.components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
    }
}

fn classify_layer(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("gui") || path_str.contains("ui") || path_str.contains("view")
        || path_str.contains("component") || path_str.contains("page")
    {
        Some("Presentation".to_string())
    } else if path_str.contains("service") || path_str.contains("manager")
        || path_str.contains("logic") || path_str.contains("core")
    {
        Some("Business Logic".to_string())
    } else if path_str.contains("db") || path_str.contains("storage")
        || path_str.contains("repository") || path_str.contains("model")
    {
        Some("Data Access".to_string())
    } else if path_str.contains("api") || path_str.contains("protocol")
        || path_str.contains("route")
    {
        Some("API/Protocol".to_string())
    } else if path_str.contains("config") || path_str.contains("infra") {
        Some("Infrastructure".to_string())
    } else {
        None
    }
}
