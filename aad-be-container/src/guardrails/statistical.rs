//! Deterministic Statistical Rigor Guardrails (Spec 44).
//!
//! Enforces four methodological gates in either `soft` (warning badges) or
//! `strict` (retry/blocking) mode over agent responses and accumulated
//! `data-engine-mcp` statistical tool envelopes:
//! 1. Assumption Violation Gate (`ASSUMPTION_UNACKNOWLEDGED`)
//! 2. Multiple-Comparison Gate (`UNADJUSTED_MULTIPLE_TESTING`)
//! 3. Reporting Completeness Gate (`INCOMPLETE_STATISTICAL_REPORT` / `UNGROUNDED_STATISTICAL_CLAIM`)
//! 4. Causal Language Gate (`UNVERIFIED_CAUSAL_CLAIM`)

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::{
    GuardrailEnforcementMode, StatisticalRigorGuardrailConfig, StatisticalRigorStatus,
    StatisticalWarning,
};

#[derive(Debug, Error)]
pub enum GuardrailError {
    #[error("invalid statistical_rigor guardrail configuration: {0}")]
    InvalidConfig(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssumptionCheckSummary {
    pub check: String,
    pub status: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatisticalEnvelopeSummary {
    pub tool: String,
    #[serde(default)]
    pub provenance_run_id: Option<String>,
    #[serde(default)]
    pub assumption_checks: Vec<AssumptionCheckSummary>,
    #[serde(default)]
    pub recommended_alternative_method: Option<String>,
    #[serde(default)]
    pub adjustment_method: Option<String>,
    #[serde(default = "default_test_count")]
    pub test_count: usize,
    #[serde(default)]
    pub is_experimental: bool,
}

fn default_test_count() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatisticalRigorEvaluation {
    pub status: StatisticalRigorStatus,
    pub warnings: Vec<StatisticalWarning>,
    pub provenance_run_ids: Vec<String>,
    pub critique_prompt: Option<String>,
}

/// Parses `guardrail_config["statistical_rigor"]` if present.
pub fn parse_statistical_rigor_config(
    guardrail_config: Option<&serde_json::Value>,
) -> Result<Option<StatisticalRigorGuardrailConfig>, GuardrailError> {
    let Some(cfg) = guardrail_config else {
        return Ok(None);
    };
    let Some(stat_val) = cfg.get("statistical_rigor") else {
        return Ok(None);
    };
    if stat_val.is_null() {
        return Ok(None);
    }
    let parsed: StatisticalRigorGuardrailConfig = serde_json::from_value(stat_val.clone())?;
    Ok(Some(parsed))
}

/// Extracts `StatisticalEnvelopeSummary` entries from execution context JSON.
/// Supports both pre-summarized `statistical_envelopes` arrays and full
/// `StatisticalResultEnvelope` objects from `data-engine-mcp`.
pub fn extract_statistical_envelopes(
    context: Option<&serde_json::Value>,
) -> Vec<StatisticalEnvelopeSummary> {
    let Some(ctx) = context else {
        return Vec::new();
    };
    let Some(items) = ctx
        .get("statistical_envelopes")
        .or_else(|| ctx.get("tool_envelopes"))
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            if let Ok(summary) = serde_json::from_value::<StatisticalEnvelopeSummary>(item.clone())
            {
                if summary.recommended_alternative_method.is_some()
                    || summary.adjustment_method.is_some()
                {
                    return Some(summary);
                }
            }

            let tool = item.get("tool")?.as_str()?.to_string();
            let provenance_run_id = item
                .get("provenance_run_id")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            let assumption_checks = item
                .get("assumption_checks")
                .and_then(|v| serde_json::from_value::<Vec<AssumptionCheckSummary>>(v.clone()).ok())
                .unwrap_or_default();
            let recommended_alternative_method = item
                .get("recommended_alternative_method")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    item.get("recommended_alternative")
                        .and_then(|alt| alt.get("method"))
                        .and_then(|m| m.as_str())
                })
                .map(ToString::to_string);
            let adjustment_method = item
                .get("adjustment_method")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    item.get("primary_result")
                        .and_then(|pr| pr.get("adjustment_method"))
                        .and_then(|m| m.as_str())
                })
                .map(ToString::to_string);
            let test_count = item
                .get("test_count")
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    item.get("primary_result")
                        .and_then(|pr| pr.get("segments"))
                        .and_then(|s| s.as_array())
                        .map(|arr| arr.len() as u64)
                })
                .map(|c| c.max(1) as usize)
                .unwrap_or(1);
            let is_experimental = item
                .get("is_experimental")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Some(StatisticalEnvelopeSummary {
                tool,
                provenance_run_id,
                assumption_checks,
                recommended_alternative_method,
                adjustment_method,
                test_count,
                is_experimental,
            })
        })
        .collect()
}

/// Evaluates all four deterministic Statistical Rigor Guardrail gates.
pub fn evaluate_statistical_rigor(
    response_text: &str,
    envelopes: &[StatisticalEnvelopeSummary],
    config: &StatisticalRigorGuardrailConfig,
) -> StatisticalRigorEvaluation {
    let provenance_run_ids: Vec<String> = envelopes
        .iter()
        .filter_map(|e| e.provenance_run_id.clone())
        .collect();

    if !config.enabled {
        return StatisticalRigorEvaluation {
            status: StatisticalRigorStatus::Passed,
            warnings: Vec::new(),
            provenance_run_ids,
            critique_prompt: None,
        };
    }

    let lower = response_text.to_lowercase();
    let normalized = normalize_tokens(&lower);
    let mut warnings = Vec::new();

    if config.require_assumption_acknowledgement {
        check_assumption_violation_gate(&normalized, envelopes, &mut warnings);
    }

    check_multiple_comparison_gate(
        envelopes,
        config.require_multiple_testing_correction_above,
        &mut warnings,
    );

    if config.require_effect_size_and_ci {
        check_reporting_completeness_gate(&lower, &normalized, envelopes, &mut warnings);
    }

    if config.flag_unverified_causal_language {
        check_causal_language_gate(&lower, envelopes, &mut warnings);
    }

    if warnings.is_empty() {
        return StatisticalRigorEvaluation {
            status: StatisticalRigorStatus::Passed,
            warnings,
            provenance_run_ids,
            critique_prompt: None,
        };
    }

    match config.mode {
        GuardrailEnforcementMode::Soft => StatisticalRigorEvaluation {
            status: StatisticalRigorStatus::Warned,
            warnings,
            provenance_run_ids,
            critique_prompt: None,
        },
        GuardrailEnforcementMode::Strict => {
            let critique_lines: Vec<String> = warnings
                .iter()
                .map(|w| format!("- [{}] {}: {}", w.gate_code, w.message, w.suggested_fix))
                .collect();
            let critique_prompt = format!(
                "Statistical Rigor Guardrail (strict mode) blocked delivery. Revise your analysis to address:\n{}",
                critique_lines.join("\n")
            );
            StatisticalRigorEvaluation {
                status: StatisticalRigorStatus::Blocked,
                warnings,
                provenance_run_ids,
                critique_prompt: Some(critique_prompt),
            }
        }
    }
}

fn normalize_tokens(text: &str) -> String {
    text.chars()
        .map(|c| if c == '_' || c == '-' { ' ' } else { c })
        .collect()
}

fn check_assumption_violation_gate(
    normalized_response: &str,
    envelopes: &[StatisticalEnvelopeSummary],
    warnings: &mut Vec<StatisticalWarning>,
) {
    for env in envelopes {
        let failed_checks: Vec<&AssumptionCheckSummary> = env
            .assumption_checks
            .iter()
            .filter(|c| c.status.eq_ignore_ascii_case("failed"))
            .collect();
        if failed_checks.is_empty() {
            continue;
        }
        let Some(alt) = env.recommended_alternative_method.as_deref() else {
            continue;
        };

        let alt_norm = normalize_tokens(&alt.to_lowercase());
        let alt_keywords = alternative_keywords(&alt_norm);
        let mentions_alt = alt_keywords
            .iter()
            .any(|kw| normalized_response.contains(kw));
        let acknowledges_failure = normalized_response.contains("assumption")
            && (normalized_response.contains("failed")
                || normalized_response.contains("violated")
                || normalized_response.contains("non normal")
                || normalized_response.contains("heteroscedastic")
                || normalized_response.contains("non stationary"));

        if !mentions_alt && !acknowledges_failure {
            let failed_names: Vec<&str> = failed_checks.iter().map(|c| c.check.as_str()).collect();
            warnings.push(StatisticalWarning {
                gate_code: "ASSUMPTION_UNACKNOWLEDGED".to_string(),
                severity: "warning".to_string(),
                message: format!(
                    "Tool '{}' reported failed assumption check(s) [{}] with recommended alternative '{}', which was not addressed in the response.",
                    env.tool,
                    failed_names.join(", "),
                    alt
                ),
                suggested_fix: format!(
                    "Acknowledge the failed assumption check(s) and report results from '{}'.",
                    alt
                ),
            });
        }
    }
}

fn alternative_keywords(alt_norm: &str) -> Vec<String> {
    let mut kws = vec![alt_norm.to_string()];
    if alt_norm.contains("mann whitney") {
        kws.push("mann whitney".to_string());
        kws.push("mwu".to_string());
    }
    if alt_norm.contains("kruskal") {
        kws.push("kruskal".to_string());
    }
    if alt_norm.contains("wilcoxon") {
        kws.push("wilcoxon".to_string());
    }
    if alt_norm.contains("fisher") {
        kws.push("fisher".to_string());
    }
    if alt_norm.contains("hc3") {
        kws.push("hc3".to_string());
        kws.push("robust standard error".to_string());
    }
    if alt_norm.contains("huber") {
        kws.push("huber".to_string());
    }
    if alt_norm.contains("differenc") {
        kws.push("differenc".to_string());
    }
    kws
}

fn check_multiple_comparison_gate(
    envelopes: &[StatisticalEnvelopeSummary],
    threshold: usize,
    warnings: &mut Vec<StatisticalWarning>,
) {
    let hyp_envelopes: Vec<&StatisticalEnvelopeSummary> = envelopes
        .iter()
        .filter(|e| e.tool == "stat_hypothesis_test")
        .collect();
    if hyp_envelopes.is_empty() {
        return;
    }

    let total_tests: usize = hyp_envelopes.iter().map(|e| e.test_count.max(1)).sum();
    if total_tests <= threshold {
        return;
    }

    let has_adjustment = hyp_envelopes.iter().any(|e| {
        e.adjustment_method
            .as_deref()
            .map(|m| {
                matches!(
                    m.to_lowercase().as_str(),
                    "fdr_bh" | "holm" | "bonferroni" | "benjamini_hochberg"
                )
            })
            .unwrap_or(false)
    });

    if !has_adjustment {
        warnings.push(StatisticalWarning {
            gate_code: "UNADJUSTED_MULTIPLE_TESTING".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Executed {} hypothesis tests (threshold: {}) without multiple-comparison adjustment.",
                total_tests, threshold
            ),
            suggested_fix: "Re-run with correction='fdr_bh', 'holm', or 'bonferroni' and report adjusted p-values.".to_string(),
        });
    }
}

fn check_reporting_completeness_gate(
    lower: &str,
    normalized: &str,
    envelopes: &[StatisticalEnvelopeSummary],
    warnings: &mut Vec<StatisticalWarning>,
) {
    let makes_sig_claim = lower.contains("p =")
        || lower.contains("p=")
        || lower.contains("p <")
        || lower.contains("p<")
        || lower.contains("p-value")
        || lower.contains("p value")
        || lower.contains("statistically significant");

    if !makes_sig_claim {
        return;
    }

    let has_backing_stat_tool = envelopes
        .iter()
        .any(|e| e.tool.starts_with("stat_") || e.tool == "algorithm_execute");

    if !has_backing_stat_tool {
        warnings.push(StatisticalWarning {
            gate_code: "UNGROUNDED_STATISTICAL_CLAIM".to_string(),
            severity: "warning".to_string(),
            message: "Response cites a p-value or statistical significance claim without a backing data-engine stat_* or algorithm_execute tool run.".to_string(),
            suggested_fix: "Execute a validated data-engine-mcp stat_* tool and ground reported statistics in its provenance_run_id.".to_string(),
        });
        return;
    }

    let mut missing = Vec::new();

    let has_sample_size = lower.contains("n =")
        || lower.contains("n=")
        || normalized.contains("sample size")
        || normalized.contains("observations")
        || normalized.contains("rows");
    if !has_sample_size {
        missing.push("sample size (n)");
    }

    let has_effect_size = normalized.contains("effect size")
        || normalized.contains("cohen")
        || normalized.contains("hedges")
        || normalized.contains("cliff")
        || normalized.contains("cramer")
        || normalized.contains("eta squared")
        || normalized.contains("odds ratio")
        || normalized.contains("r squared")
        || lower.contains("r²")
        || lower.contains("r^2");
    if !has_effect_size {
        missing.push("effect size");
    }

    let has_ci = lower.contains("ci")
        || normalized.contains("confidence interval")
        || normalized.contains("prediction interval");
    if !has_ci {
        missing.push("confidence interval");
    }

    let expected_prov_ids: Vec<&str> = envelopes
        .iter()
        .filter_map(|e| e.provenance_run_id.as_deref())
        .collect();
    let has_provenance = if expected_prov_ids.is_empty() {
        normalized.contains("provenance")
    } else {
        expected_prov_ids
            .iter()
            .any(|id| lower.contains(&id.to_lowercase()))
    };
    if !has_provenance {
        missing.push("provenance_run_id");
    }

    if !missing.is_empty() {
        warnings.push(StatisticalWarning {
            gate_code: "INCOMPLETE_STATISTICAL_REPORT".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Statistical significance claim omits required disclosure(s): {}.",
                missing.join(", ")
            ),
            suggested_fix: format!(
                "Include {} alongside every reported p-value.",
                missing.join(", ")
            ),
        });
    }
}

fn check_causal_language_gate(
    lower: &str,
    envelopes: &[StatisticalEnvelopeSummary],
    warnings: &mut Vec<StatisticalWarning>,
) {
    let observational_only = !envelopes.is_empty()
        && envelopes.iter().all(|e| {
            !e.is_experimental
                && matches!(
                    e.tool.as_str(),
                    "stat_correlation_vif" | "stat_regression_fit"
                )
        });
    if !observational_only {
        return;
    }

    let causal_phrases = [
        " causes ",
        " caused by ",
        " proves that ",
        " directly drives ",
        " proves impact",
        " causally increases ",
        " causally decreases ",
    ];
    let padded = format!(" {} ", lower);
    let uses_causal_claim = causal_phrases.iter().any(|p| padded.contains(p));
    let has_hedging = lower.contains("does not imply causation")
        || lower.contains("not causal")
        || lower.contains("observational");

    if uses_causal_claim && !has_hedging {
        warnings.push(StatisticalWarning {
            gate_code: "UNVERIFIED_CAUSAL_CLAIM".to_string(),
            severity: "warning".to_string(),
            message: "Response uses assertive causal phrasing based solely on observational correlation/regression tools.".to_string(),
            suggested_fix: "Replace causal assertions with associative phrasing ('associated with', 'correlated with') and note observational study limitations.".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_soft_config() -> StatisticalRigorGuardrailConfig {
        StatisticalRigorGuardrailConfig {
            enabled: true,
            mode: GuardrailEnforcementMode::Soft,
            require_assumption_acknowledgement: true,
            require_multiple_testing_correction_above: 1,
            require_effect_size_and_ci: true,
            flag_unverified_causal_language: true,
        }
    }

    #[test]
    fn test_assumption_violation_gate_flags_ignored_normality_failure_and_passes_when_mwu_reported() {
        let envelopes = vec![StatisticalEnvelopeSummary {
            tool: "stat_hypothesis_test".to_string(),
            provenance_run_id: Some("prov-001".to_string()),
            assumption_checks: vec![AssumptionCheckSummary {
                check: "normality_shapiro_or_dagostino".to_string(),
                status: "failed".to_string(),
                message: Some("Heavy right skew".to_string()),
            }],
            recommended_alternative_method: Some("mann_whitney_u".to_string()),
            adjustment_method: None,
            test_count: 1,
            is_experimental: true,
        }];

        let bad_resp = "Welch t-test p = 0.016, n = 500, Hedges' g = 0.04, 95% CI [0.4, 4.1], provenance prov-001.";
        let eval_bad = evaluate_statistical_rigor(bad_resp, &envelopes, &default_soft_config());
        assert_eq!(eval_bad.status, StatisticalRigorStatus::Warned);
        assert!(
            eval_bad
                .warnings
                .iter()
                .any(|w| w.gate_code == "ASSUMPTION_UNACKNOWLEDGED")
        );

        let good_resp = "Normality failed; reporting Mann-Whitney U (p = 0.412, n = 500, Cliff's delta = -0.011, 95% CI [-0.04, 0.02], provenance prov-001).";
        let eval_good = evaluate_statistical_rigor(good_resp, &envelopes, &default_soft_config());
        assert_eq!(eval_good.status, StatisticalRigorStatus::Passed);
        assert!(eval_good.warnings.is_empty());
    }

    #[test]
    fn test_multiple_comparison_gate_detects_20_segment_unadjusted_p_hacking() {
        let unadjusted = vec![StatisticalEnvelopeSummary {
            tool: "stat_hypothesis_test".to_string(),
            provenance_run_id: Some("prov-002".to_string()),
            assumption_checks: vec![],
            recommended_alternative_method: None,
            adjustment_method: None,
            test_count: 20,
            is_experimental: true,
        }];
        let resp = "Segment 7 had p = 0.031 (n = 200, Cohen's d = 0.25, 95% CI [0.02, 0.48], prov-002).";
        let eval_bad = evaluate_statistical_rigor(resp, &unadjusted, &default_soft_config());
        assert_eq!(eval_bad.status, StatisticalRigorStatus::Warned);
        assert!(
            eval_bad
                .warnings
                .iter()
                .any(|w| w.gate_code == "UNADJUSTED_MULTIPLE_TESTING")
        );

        let mut adjusted = unadjusted;
        adjusted[0].adjustment_method = Some("fdr_bh".to_string());
        let eval_good = evaluate_statistical_rigor(resp, &adjusted, &default_soft_config());
        assert_eq!(eval_good.status, StatisticalRigorStatus::Passed);
    }

    #[test]
    fn test_reporting_completeness_gate_requires_effect_size_ci_n_and_provenance() {
        let envelopes = vec![StatisticalEnvelopeSummary {
            tool: "stat_hypothesis_test".to_string(),
            provenance_run_id: Some("prov-abc-123".to_string()),
            assumption_checks: vec![],
            recommended_alternative_method: None,
            adjustment_method: None,
            test_count: 1,
            is_experimental: true,
        }];

        let bare = "The result is statistically significant (p = 0.012)!";
        let eval_bad = evaluate_statistical_rigor(bare, &envelopes, &default_soft_config());
        assert_eq!(eval_bad.status, StatisticalRigorStatus::Warned);
        assert!(
            eval_bad
                .warnings
                .iter()
                .any(|w| w.gate_code == "INCOMPLETE_STATISTICAL_REPORT")
        );

        let full = "Treatment lift (n = 1200, Cohen's d = 0.32, 95% CI [0.18, 0.46], p = 0.012, provenance_run_id: prov-abc-123).";
        let eval_good = evaluate_statistical_rigor(full, &envelopes, &default_soft_config());
        assert_eq!(eval_good.status, StatisticalRigorStatus::Passed);
    }

    #[test]
    fn test_causal_language_gate_flags_causal_claims_on_observational_correlation() {
        let envelopes = vec![StatisticalEnvelopeSummary {
            tool: "stat_correlation_vif".to_string(),
            provenance_run_id: Some("prov-corr-1".to_string()),
            assumption_checks: vec![],
            recommended_alternative_method: None,
            adjustment_method: Some("fdr_bh".to_string()),
            test_count: 1,
            is_experimental: false,
        }];

        let causal = "Marketing spend causes higher retention in our dataset.";
        let eval_bad = evaluate_statistical_rigor(causal, &envelopes, &default_soft_config());
        assert_eq!(eval_bad.status, StatisticalRigorStatus::Warned);
        assert!(
            eval_bad
                .warnings
                .iter()
                .any(|w| w.gate_code == "UNVERIFIED_CAUSAL_CLAIM")
        );

        let associative = "Marketing spend is positively associated with retention (observational correlation; does not imply causation).";
        let eval_good = evaluate_statistical_rigor(associative, &envelopes, &default_soft_config());
        assert_eq!(eval_good.status, StatisticalRigorStatus::Passed);
    }

    #[test]
    fn test_soft_mode_attaches_warnings_while_strict_mode_blocks_or_retries() {
        let envelopes = vec![StatisticalEnvelopeSummary {
            tool: "stat_hypothesis_test".to_string(),
            provenance_run_id: Some("prov-strict-1".to_string()),
            assumption_checks: vec![],
            recommended_alternative_method: None,
            adjustment_method: None,
            test_count: 1,
            is_experimental: true,
        }];
        let flawed = "p = 0.004, statistically significant!";

        let soft_eval = evaluate_statistical_rigor(flawed, &envelopes, &default_soft_config());
        assert_eq!(soft_eval.status, StatisticalRigorStatus::Warned);
        assert!(soft_eval.critique_prompt.is_none());

        let mut strict_cfg = default_soft_config();
        strict_cfg.mode = GuardrailEnforcementMode::Strict;
        let strict_eval = evaluate_statistical_rigor(flawed, &envelopes, &strict_cfg);
        assert_eq!(strict_eval.status, StatisticalRigorStatus::Blocked);
        assert!(strict_eval.critique_prompt.is_some());
    }
}
