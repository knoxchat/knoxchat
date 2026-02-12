//! Code Clone and Similarity Detection
//!
//! This module detects duplicated code patterns and similar logic across the codebase,
//! helping identify refactoring opportunities and maintain code quality.

use crate::error::Result;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Code clone detector
pub struct CloneDetector {
    config: CloneDetectionConfig,
}

/// Configuration for clone detection
#[derive(Debug, Clone)]
pub struct CloneDetectionConfig {
    pub min_tokens: usize,
    pub min_lines: usize,
    pub similarity_threshold: f64,
    pub enable_type1: bool, // Exact clones
    pub enable_type2: bool, // Renamed clones
    pub enable_type3: bool, // Near-miss clones
    pub enable_type4: bool, // Semantic clones
}

impl Default for CloneDetectionConfig {
    fn default() -> Self {
        Self {
            min_tokens: 50,
            min_lines: 6,
            similarity_threshold: 0.85,
            enable_type1: true,
            enable_type2: true,
            enable_type3: true,
            enable_type4: false, // Requires more advanced analysis
        }
    }
}

/// Detected code clone
#[derive(Debug, Clone)]
pub struct CodeClone {
    pub clone_type: CloneType,
    pub fragments: Vec<CodeFragment>,
    pub similarity_score: f64,
    pub refactoring_suggestion: Option<RefactoringSuggestion>,
}

/// Type of code clone
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CloneType {
    Type1Exact,    // Exact copies
    Type2Renamed,  // Same structure, renamed identifiers
    Type3NearMiss, // Similar with small differences
    Type4Semantic, // Same functionality, different implementation
}

/// Code fragment involved in clone
#[derive(Debug, Clone)]
pub struct CodeFragment {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub normalized_content: String,
    pub token_hash: u64,
}

/// Refactoring suggestion
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub strategy: RefactoringStrategy,
    pub estimated_benefit: EstimatedBenefit,
    pub implementation_notes: Vec<String>,
}

/// Refactoring strategy
#[derive(Debug, Clone, PartialEq)]
pub enum RefactoringStrategy {
    ExtractFunction,
    ExtractClass,
    ExtractModule,
    ParameterizeCode,
    UseCommonUtility,
}

/// Estimated benefit of refactoring
#[derive(Debug, Clone)]
pub struct EstimatedBenefit {
    pub lines_saved: usize,
    pub maintainability_improvement: f64,
    pub test_coverage_improvement: f64,
}

/// Clone detection result
#[derive(Debug, Clone)]
pub struct CloneDetectionResult {
    pub total_clones: usize,
    pub clones_by_type: HashMap<CloneType, usize>,
    pub code_duplication_percentage: f64,
    pub top_clones: Vec<CodeClone>,
    pub refactoring_opportunities: Vec<RefactoringOpportunity>,
}

/// Refactoring opportunity
#[derive(Debug, Clone)]
pub struct RefactoringOpportunity {
    pub description: String,
    pub affected_files: Vec<String>,
    pub estimated_effort: EffortLevel,
    pub estimated_impact: ImpactLevel,
    pub priority: Priority,
}

/// Effort level
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Impact level
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Priority
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl CloneDetector {
    /// Create a new clone detector
    pub fn new() -> Self {
        Self {
            config: CloneDetectionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CloneDetectionConfig) -> Self {
        Self { config }
    }

    /// Detect clones in codebase
    pub fn detect_clones(&mut self, files: &[(String, String)]) -> Result<CloneDetectionResult> {
        let mut all_clones = Vec::new();
        let mut clones_by_type: HashMap<CloneType, usize> = HashMap::new();

        // Extract code fragments from files
        let fragments = self.extract_fragments(files)?;

        // Type 1: Exact clones
        if self.config.enable_type1 {
            let type1_clones = self.find_exact_clones(&fragments)?;
            *clones_by_type.entry(CloneType::Type1Exact).or_insert(0) += type1_clones.len();
            all_clones.extend(type1_clones);
        }

        // Type 2: Renamed clones
        if self.config.enable_type2 {
            let type2_clones = self.find_renamed_clones(&fragments)?;
            *clones_by_type.entry(CloneType::Type2Renamed).or_insert(0) += type2_clones.len();
            all_clones.extend(type2_clones);
        }

        // Type 3: Near-miss clones
        if self.config.enable_type3 {
            let type3_clones = self.find_near_miss_clones(&fragments)?;
            *clones_by_type.entry(CloneType::Type3NearMiss).or_insert(0) += type3_clones.len();
            all_clones.extend(type3_clones);
        }

        // Type 4: Semantic clones (requires deeper analysis)
        if self.config.enable_type4 {
            let type4_clones = self.find_semantic_clones(&fragments)?;
            *clones_by_type.entry(CloneType::Type4Semantic).or_insert(0) += type4_clones.len();
            all_clones.extend(type4_clones);
        }

        // Calculate metrics
        let total_lines: usize = files
            .iter()
            .map(|(_, content)| content.lines().count())
            .sum();
        let duplicated_lines: usize = all_clones
            .iter()
            .flat_map(|c| &c.fragments)
            .map(|f| f.end_line - f.start_line)
            .sum();

        let duplication_percentage = if total_lines > 0 {
            (duplicated_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        // Generate refactoring opportunities
        let refactoring_opportunities = self.generate_refactoring_opportunities(&all_clones)?;

        // Sort and take top clones
        all_clones.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_clones = all_clones.iter().take(20).cloned().collect();

        Ok(CloneDetectionResult {
            total_clones: all_clones.len(),
            clones_by_type,
            code_duplication_percentage: duplication_percentage,
            top_clones,
            refactoring_opportunities,
        })
    }

    /// Extract code fragments from files
    fn extract_fragments(&self, files: &[(String, String)]) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();

        for (file_path, content) in files {
            let lines: Vec<&str> = content.lines().collect();

            // Extract fragments using sliding window
            for start in 0..lines.len() {
                for end in (start + self.config.min_lines)..=lines.len().min(start + 100) {
                    let fragment_content: String = lines[start..end].join("\n");

                    if self.is_valid_fragment(&fragment_content) {
                        let normalized = self.normalize_code(&fragment_content);
                        let hash = self.compute_hash(&normalized);

                        fragments.push(CodeFragment {
                            file_path: file_path.clone(),
                            start_line: start + 1,
                            end_line: end + 1,
                            content: fragment_content,
                            normalized_content: normalized,
                            token_hash: hash,
                        });
                    }
                }
            }
        }

        Ok(fragments)
    }

    /// Find exact clones (Type 1)
    fn find_exact_clones(&self, fragments: &[CodeFragment]) -> Result<Vec<CodeClone>> {
        let mut clones = Vec::new();
        let mut hash_map: HashMap<u64, Vec<&CodeFragment>> = HashMap::new();

        // Group by hash
        for fragment in fragments {
            hash_map
                .entry(fragment.token_hash)
                .or_insert_with(Vec::new)
                .push(fragment);
        }

        // Find groups with multiple fragments
        for (_, group) in hash_map.iter() {
            if group.len() >= 2 {
                clones.push(CodeClone {
                    clone_type: CloneType::Type1Exact,
                    fragments: group.iter().map(|f| (*f).clone()).collect(),
                    similarity_score: 1.0,
                    refactoring_suggestion: Some(
                        self.suggest_refactoring(&CloneType::Type1Exact, group),
                    ),
                });
            }
        }

        Ok(clones)
    }

    /// Find renamed clones (Type 2)
    fn find_renamed_clones(&self, fragments: &[CodeFragment]) -> Result<Vec<CodeClone>> {
        let mut clones = Vec::new();

        // Compare fragments pairwise with identifier normalization
        for i in 0..fragments.len() {
            for j in (i + 1)..fragments.len() {
                let similarity = self.compute_structural_similarity(&fragments[i], &fragments[j]);

                if similarity >= self.config.similarity_threshold {
                    clones.push(CodeClone {
                        clone_type: CloneType::Type2Renamed,
                        fragments: vec![fragments[i].clone(), fragments[j].clone()],
                        similarity_score: similarity,
                        refactoring_suggestion: Some(self.suggest_refactoring(
                            &CloneType::Type2Renamed,
                            &[&fragments[i], &fragments[j]],
                        )),
                    });
                }
            }
        }

        Ok(clones)
    }

    /// Find near-miss clones (Type 3)
    fn find_near_miss_clones(&self, fragments: &[CodeFragment]) -> Result<Vec<CodeClone>> {
        let mut clones = Vec::new();

        for i in 0..fragments.len() {
            for j in (i + 1)..fragments.len() {
                let similarity = self.compute_fuzzy_similarity(&fragments[i], &fragments[j]);

                if similarity >= self.config.similarity_threshold * 0.9
                    && similarity < self.config.similarity_threshold
                {
                    clones.push(CodeClone {
                        clone_type: CloneType::Type3NearMiss,
                        fragments: vec![fragments[i].clone(), fragments[j].clone()],
                        similarity_score: similarity,
                        refactoring_suggestion: Some(self.suggest_refactoring(
                            &CloneType::Type3NearMiss,
                            &[&fragments[i], &fragments[j]],
                        )),
                    });
                }
            }
        }

        Ok(clones)
    }

    /// Find semantic clones (Type 4)
    fn find_semantic_clones(&self, _fragments: &[CodeFragment]) -> Result<Vec<CodeClone>> {
        // Semantic clone detection requires deeper analysis (AST comparison, data flow, etc.)
        // This is a placeholder for future ML-based implementation
        Ok(Vec::new())
    }

    /// Check if fragment is valid for analysis
    fn is_valid_fragment(&self, content: &str) -> bool {
        let lines = content.lines().count();
        let tokens = content.split_whitespace().count();

        lines >= self.config.min_lines
            && tokens >= self.config.min_tokens
            && !content.trim().is_empty()
    }

    /// Normalize code for comparison
    fn normalize_code(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Compute hash for code fragment
    fn compute_hash(&self, code: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute structural similarity
    fn compute_structural_similarity(&self, frag1: &CodeFragment, frag2: &CodeFragment) -> f64 {
        // Simple token-based similarity
        let tokens1: HashSet<_> = frag1.normalized_content.split_whitespace().collect();
        let tokens2: HashSet<_> = frag2.normalized_content.split_whitespace().collect();

        let intersection = tokens1.intersection(&tokens2).count();
        let union = tokens1.union(&tokens2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Compute fuzzy similarity
    fn compute_fuzzy_similarity(&self, frag1: &CodeFragment, frag2: &CodeFragment) -> f64 {
        // Levenshtein-like similarity
        let lines1: Vec<_> = frag1.normalized_content.lines().collect();
        let lines2: Vec<_> = frag2.normalized_content.lines().collect();

        let mut matching_lines = 0;
        for line1 in &lines1 {
            for line2 in &lines2 {
                if line1 == line2 {
                    matching_lines += 1;
                    break;
                }
            }
        }

        let total_lines = lines1.len().max(lines2.len());
        if total_lines == 0 {
            0.0
        } else {
            matching_lines as f64 / total_lines as f64
        }
    }

    /// Suggest refactoring based on clone type
    fn suggest_refactoring(
        &self,
        clone_type: &CloneType,
        fragments: &[&CodeFragment],
    ) -> RefactoringSuggestion {
        let strategy = match clone_type {
            CloneType::Type1Exact | CloneType::Type2Renamed => RefactoringStrategy::ExtractFunction,
            CloneType::Type3NearMiss => RefactoringStrategy::ParameterizeCode,
            CloneType::Type4Semantic => RefactoringStrategy::UseCommonUtility,
        };

        let total_lines: usize = fragments.iter().map(|f| f.end_line - f.start_line).sum();

        RefactoringSuggestion {
            strategy,
            estimated_benefit: EstimatedBenefit {
                lines_saved: total_lines / 2,
                maintainability_improvement: 0.3,
                test_coverage_improvement: 0.1,
            },
            implementation_notes: vec![
                "Extract common logic into shared function".to_string(),
                "Update all call sites".to_string(),
                "Add appropriate tests".to_string(),
            ],
        }
    }

    /// Generate refactoring opportunities
    fn generate_refactoring_opportunities(
        &self,
        clones: &[CodeClone],
    ) -> Result<Vec<RefactoringOpportunity>> {
        let mut opportunities = Vec::new();

        for clone in clones {
            let affected_files: Vec<_> = clone
                .fragments
                .iter()
                .map(|f| f.file_path.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let effort = if clone.fragments.len() <= 2 {
                EffortLevel::Low
            } else if clone.fragments.len() <= 5 {
                EffortLevel::Medium
            } else {
                EffortLevel::High
            };

            let impact = if clone.similarity_score > 0.95 {
                ImpactLevel::High
            } else if clone.similarity_score > 0.85 {
                ImpactLevel::Medium
            } else {
                ImpactLevel::Low
            };

            opportunities.push(RefactoringOpportunity {
                description: format!(
                    "Refactor {} clones with {:.1}% similarity",
                    clone.fragments.len(),
                    clone.similarity_score * 100.0
                ),
                affected_files,
                estimated_effort: effort,
                estimated_impact: impact,
                priority: self.calculate_priority(&clone),
            });
        }

        // Sort by priority
        opportunities.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        Ok(opportunities)
    }

    /// Calculate priority
    fn calculate_priority(&self, clone: &CodeClone) -> Priority {
        if clone.fragments.len() >= 5 && clone.similarity_score > 0.95 {
            Priority::Critical
        } else if clone.fragments.len() >= 3 && clone.similarity_score > 0.90 {
            Priority::High
        } else if clone.similarity_score > 0.85 {
            Priority::Medium
        } else {
            Priority::Low
        }
    }
}

impl Default for CloneDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = CloneDetector::new();
        assert_eq!(detector.config.min_lines, 6);
        assert_eq!(detector.config.similarity_threshold, 0.85);
    }

    #[test]
    fn test_normalize_code() {
        let detector = CloneDetector::new();
        let code = "  function test() {\n    return 42;\n  }  ";
        let normalized = detector.normalize_code(code);
        assert!(!normalized.contains("  "));
    }

    #[test]
    fn test_is_valid_fragment() {
        let detector = CloneDetector::new();
        let valid = "fn test() {\n    let x = 1;\n    let y = 2;\n    let z = 3;\n    let a = 4;\n    let b = 5;\n}";
        assert!(detector.is_valid_fragment(valid));

        let invalid = "fn test() { }";
        assert!(!detector.is_valid_fragment(invalid));
    }
}
