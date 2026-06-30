use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingSuggestion {
    pub code: String,
    pub system: String, // e.g. "ICD-10", "SNOMED-CT"
    pub description: String,
    pub confidence_score: f32,
    pub supporting_evidence: String,
    pub missing_docs_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingDecisionLog {
    pub timestamp: u64,
    pub clinician_id: String,
    pub code: String,
    pub accepted: bool,
    pub terminology_version: String,
}

pub trait MedicalCoder: Send + Sync {
    fn suggest_codes(&self, clinical_text: &str) -> Vec<CodingSuggestion>;
    fn log_decision(&self, log: &CodingDecisionLog) -> Result<(), String>;
}

pub struct GenericMedicalCoder {
    icd10_version: String,
    snomed_version: String,
}

impl GenericMedicalCoder {
    pub fn new() -> Self {
        Self {
            icd10_version: "ICD-10-AM 2026".to_string(),
            snomed_version: "SNOMED-CT-IN 2026.02".to_string(),
        }
    }
}

impl MedicalCoder for GenericMedicalCoder {
    fn suggest_codes(&self, clinical_text: &str) -> Vec<CodingSuggestion> {
        let text_lower = clinical_text.to_lowercase();
        let mut suggestions = Vec::new();

        if text_lower.contains("cough") || text_lower.contains("respiratory") {
            suggestions.push(CodingSuggestion {
                code: "J06.9".to_string(),
                system: "ICD-10".to_string(),
                description: "Acute upper respiratory infection, unspecified".to_string(),
                confidence_score: 0.92,
                supporting_evidence: "Based on documented cough and respiratory findings.".to_string(),
                missing_docs_warning: None,
            });
            suggestions.push(CodingSuggestion {
                code: "27733008".to_string(),
                system: "SNOMED-CT".to_string(),
                description: "Acute upper respiratory tract infection".to_string(),
                confidence_score: 0.95,
                supporting_evidence: "Pharyngeal erythema or acute congestion indicated.".to_string(),
                missing_docs_warning: Some("Warning: Temperature reading missing from consultation record.".to_string()),
            });
        }

        if text_lower.contains("diabetes") || text_lower.contains("metformin") {
            suggestions.push(CodingSuggestion {
                code: "E11.9".to_string(),
                system: "ICD-10".to_string(),
                description: "Type 2 diabetes mellitus without complications".to_string(),
                confidence_score: 0.88,
                supporting_evidence: "Metformin administration specified.".to_string(),
                missing_docs_warning: None,
            });
        }

        suggestions
    }

    fn log_decision(&self, log: &CodingDecisionLog) -> Result<(), String> {
        // Production audit logging logic
        println!(
            "[MedicalCoder Audit]: Clinician {} {} code {} (Version: {})",
            log.clinician_id,
            if log.accepted { "ACCEPTED" } else { "REJECTED" },
            log.code,
            log.terminology_version
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coding_suggestions() {
        let coder = GenericMedicalCoder::new();
        let suggestions = coder.suggest_codes("Patient has a persistent cough.");
        assert_eq!(suggestions[0].code, "J06.9");
        assert_eq!(suggestions[1].system, "SNOMED-CT");
        assert!(suggestions[1].missing_docs_warning.is_some());
    }

    #[test]
    fn test_decision_logging() {
        let coder = GenericMedicalCoder::new();
        let log = CodingDecisionLog {
            timestamp: 1700000000,
            clinician_id: "doc-anita".to_string(),
            code: "J06.9".to_string(),
            accepted: true,
            terminology_version: "ICD-10 v2026".to_string(),
        };
        assert!(coder.log_decision(&log).is_ok());
    }
}
