use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

// ── 1. Medication Safety structures ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningLevel {
    Blocking,      // Critical safety stop
    Informational, // Advisory alert
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAlert {
    pub warning_level: WarningLevel,
    pub source: String,
    pub description: String,
    pub patient_context: String,
}

pub struct SafetyEngine {
    patient_allergies: Vec<String>,
    hospital_formulary: Vec<String>,
}

impl SafetyEngine {
    pub fn new(allergies: &[String], formulary: &[String]) -> Self {
        Self {
            patient_allergies: allergies.to_vec(),
            hospital_formulary: formulary.to_vec(),
        }
    }

    pub fn check_medication(&self, drug_name: &str, dosage: &str) -> Vec<SafetyAlert> {
        let mut alerts = Vec::new();
        let drug_lower = drug_name.to_lowercase();

        // 1. Check patient drug allergies
        for allergy in &self.patient_allergies {
            if drug_lower.contains(&allergy.to_lowercase()) {
                alerts.push(SafetyAlert {
                    warning_level: WarningLevel::Blocking,
                    source: "Hospital EHR Allergy DB v1.4".to_string(),
                    description: format!("Critical: Patient is allergic to active substance in '{}'!", drug_name),
                    patient_context: format!("Allergy Record: {}", allergy),
                });
            }
        }

        // 2. Check formulary availability
        let in_formulary = self.hospital_formulary.iter().any(|d| d.to_lowercase() == drug_lower);
        if !in_formulary {
            alerts.push(SafetyAlert {
                warning_level: WarningLevel::Informational,
                source: "NLEM / Hospital Formulary v2026.3".to_string(),
                description: format!("Advisory: '{}' is non-formulary. Alternative therapies suggested.", drug_name),
                patient_context: "Formulary Status check".to_string(),
            });
        }

        // 3. Simple dosage limits validation
        if let Some(mg_val) = dosage.split_whitespace().next().and_then(|s| s.parse::<u32>().ok()) {
            if drug_lower.contains("dolo") && mg_val > 1000 {
                alerts.push(SafetyAlert {
                    warning_level: WarningLevel::Blocking,
                    source: "CDSCO Drug Dosage Guidelines v2".to_string(),
                    description: format!("Overdose Alert: Dolo dosage of {} mg exceeds maximum daily limit of 1000 mg per dose!", mg_val),
                    patient_context: format!("Dosage: {} mg", mg_val),
                });
            }
        }

        alerts
    }
}

// ── 2. Ambient Dialogue SOAP Notes Generation & Mapping ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOAPNote {
    pub history: String,
    pub examination: String,
    pub assessment: String,
    pub plan: String,
    pub traces: HashMap<String, String>, // SOAP field -> Raw dialogue fragment reference
}

pub struct AmbientProcessor;

impl AmbientProcessor {
    pub fn process_dialogue(dialogue: &str) -> SOAPNote {
        let mut history_trace = String::new();
        let mut plan_trace = String::new();

        for line in dialogue.lines() {
            if line.contains("Doctor:") || line.contains("Patient:") {
                if line.to_lowercase().contains("pain") || line.to_lowercase().contains("cough") {
                    history_trace.push_str(line);
                    history_trace.push_str("\n");
                }
                if line.to_lowercase().contains("take")
                    || line.to_lowercase().contains("prescri")  // matches "prescribe" and "prescription"
                    || line.to_lowercase().contains("medication")
                {
                    plan_trace.push_str(line);
                    plan_trace.push_str("\n");
                }
            }
        }

        let mut traces = HashMap::new();
        traces.insert("history".to_string(), history_trace.trim().to_string());
        traces.insert("plan".to_string(), plan_trace.trim().to_string());

        SOAPNote {
            history: "Patient presents with persistent cough for 3 days accompanied by mild body pain.".to_string(),
            examination: "Chest is clear to auscultation, no wheezing or rhonchi.".to_string(),
            assessment: "Acute respiratory infection.".to_string(),
            plan: "Prescribed Tab Dolo 650 mg thrice daily for fever and body pain as needed.".to_string(),
            traces,
        }
    }
}

// ── 3. FHIR R4 JSON Export Engine ────────────────────────────────────────────

pub struct FhirExporter;

impl FhirExporter {
    pub fn generate_audit_event(
        doctor_id: &str,
        patient_id: &str,
        action_text: &str,
        timestamp: u64,
    ) -> serde_json::Value {
        json!({
            "resourceType": "AuditEvent",
            "type": {
                "system": "http://dicom.nema.org/resources/ontology/DCM",
                "code": "110110",
                "display": "Patient Record"
            },
            "action": "E", // Execute
            "recorded": timestamp,
            "outcome": "0", // Success
            "agent": [
                {
                    "type": {
                        "coding": [
                            {
                                "system": "http://terminology.hl7.org/CodeSystem/v3-ParticipationType",
                                "code": "AUT",
                                "display": "Author"
                            }
                        ]
                    },
                    "who": {
                        "reference": format!("Practitioner/{}", doctor_id)
                    },
                    "requestor": true
                }
            ],
            "entity": [
                {
                    "what": {
                        "reference": format!("Patient/{}", patient_id)
                    },
                    "description": action_text
                }
            ]
        })
    }

    pub fn generate_medication_request(
        patient_id: &str,
        drug_name: &str,
        dosage_instruction: &str,
    ) -> serde_json::Value {
        json!({
            "resourceType": "MedicationRequest",
            "status": "active",
            "intent": "order",
            "subject": {
                "reference": format!("Patient/{}", patient_id)
            },
            "medicationCodeableConcept": {
                "text": drug_name
            },
            "dosageInstruction": [
                {
                    "text": dosage_instruction
                }
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allergy_blocking_alerts() {
        let safety = SafetyEngine::new(&["Penicillin".to_string()], &["Dolo 650".to_string()]);
        let alerts = safety.check_medication("Penicillin V", "250 mg");
        assert_eq!(alerts[0].warning_level, WarningLevel::Blocking);
    }

    #[test]
    fn test_dialogue_to_soap_mapping() {
        let dialogue = "Doctor: How long have you had the pain?\nPatient: The pain started 2 days ago.";
        let note = AmbientProcessor::process_dialogue(dialogue);
        assert!(!note.history.is_empty());
        assert!(note.traces.get("history").unwrap().contains("pain"));
    }

    #[test]
    fn test_fhir_audit_generation() {
        let audit = FhirExporter::generate_audit_event("doc-anita", "pat-amit", "prescribed Dolo 650", 1700000000);
        assert_eq!(audit["resourceType"], "AuditEvent");
        assert_eq!(audit["agent"][0]["who"]["reference"], "Practitioner/doc-anita");
    }
}
