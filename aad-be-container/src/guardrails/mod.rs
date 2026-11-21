pub mod statistical;

pub use statistical::{
    AssumptionCheckSummary, GuardrailError, StatisticalEnvelopeSummary,
    StatisticalRigorEvaluation, evaluate_statistical_rigor, extract_statistical_envelopes,
    parse_statistical_rigor_config,
};
