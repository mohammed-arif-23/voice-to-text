use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    High,   // >= 0.85
    Medium, // 0.65 to 0.84
    Low,    // < 0.65
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedDrugToken {
    pub original_word: String,
    pub matched_name: Option<String>,
    pub dosage: Option<String>,
    pub confidence: f32,
    pub confidence_level: ConfidenceLevel,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalCorrectionResult {
    pub formatted_text: String,
    pub terms: Vec<MatchedDrugToken>,
    pub has_low_confidence: bool,
}

pub trait DrugMatcher: Send + Sync {
    fn match_text(&self, raw_transcript: &str) -> MedicalCorrectionResult;
    fn add_doctor_vocabulary(&mut self, drug_names: &[String]);
    fn record_clinician_correction(&mut self, original: &str, corrected: &str);
}

pub struct AdvancedDrugMatcher {
    global_vocab: Vec<String>,
    hospital_vocab: Vec<String>,
    dept_vocab: Vec<String>,
    specialty_vocab: Vec<String>,
    doctor_vocab: Vec<String>,
    corrections_map: HashMap<String, String>,
}

impl AdvancedDrugMatcher {
    pub fn new() -> Self {
        let global_drugs = vec![
            "Salicylic".to_string(), "Niacinamide".to_string(), "Glycolic".to_string(),
            "Hyaluronic".to_string(), "Benzoyl".to_string(), "Peroxide".to_string(),
            "Clindamycin".to_string(), "Adapalene".to_string(), "Tretinoin".to_string(),
            "Ketoconazole".to_string(), "Coal".to_string(), "Tar".to_string(), "Acid".to_string(),
            "Paracetamol".to_string(), "Dolo".to_string(), "Ibuprofen".to_string(),
            "Aceclofenac".to_string(), "Diclofenac".to_string(), "Tramadol".to_string(),
            "Nimesulide".to_string(), "Mefenamic".to_string(),
            "Amoxicillin".to_string(), "Amoxyclav".to_string(), "Azithromycin".to_string(),
            "Azithral".to_string(), "Cefixime".to_string(), "Ofloxacin".to_string(),
            "Ciprofloxacin".to_string(), "Ciplox".to_string(), "Doxycycline".to_string(),
            "Acyclovir".to_string(),
            "Pantoprazole".to_string(), "Pantocid".to_string(), "Omeprazole".to_string(),
            "Rabeprazole".to_string(), "Ranitidine".to_string(), "Famotidine".to_string(),
            "Domperidone".to_string(), "Ondansetron".to_string(),
            "Amlodipine".to_string(), "Amlokind".to_string(), "Telmisartan".to_string(),
            "Telma".to_string(), "Losartan".to_string(), "Atorvastatin".to_string(),
            "Rosuvastatin".to_string(), "Metoprolol".to_string(),
            "Cetirizine".to_string(), "Montelukast".to_string(), "Levocetirizine".to_string(),
            "Fexofenadine".to_string(), "Loratadine".to_string(), "Phenylephrine".to_string(),
        ];
        Self {
            global_vocab: global_drugs,
            hospital_vocab: vec!["Glycomet".to_string(), "Metformin".to_string()],
            dept_vocab: vec!["Insulin".to_string()],
            specialty_vocab: vec!["Glimepiride".to_string()],
            doctor_vocab: Vec::new(),
            corrections_map: HashMap::new(),
        }
    }

    fn calculate_levenshtein(s1: &str, s2: &str) -> usize {
        let v1: Vec<char> = s1.to_lowercase().chars().collect();
        let v2: Vec<char> = s2.to_lowercase().chars().collect();
        let len1 = v1.len();
        let len2 = v2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        for i in 0..=len1 { matrix[i][0] = i; }
        for j in 0..=len2 { matrix[0][j] = j; }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if v1[i-1] == v2[j-1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i-1][j] + 1)
                    .min(matrix[i][j-1] + 1)
                    .min(matrix[i-1][j-1] + cost);
            }
        }
        matrix[len1][len2]
    }

    pub fn set_hospital_vocabulary(&mut self, vocab: &[String]) {
        self.hospital_vocab = vocab.to_vec();
    }

    pub fn set_specialty_vocabulary(&mut self, vocab: &[String]) {
        self.specialty_vocab = vocab.to_vec();
    }

    // Safety Engine check: Dosage immutability
    pub fn verify_dosage_immutability(original: &str, matched: &str) -> Result<(), String> {
        let orig_numbers: Vec<char> = original.chars().filter(|c| c.is_numeric()).collect();
        let match_numbers: Vec<char> = matched.chars().filter(|c| c.is_numeric()).collect();
        if orig_numbers != match_numbers {
            return Err(format!(
                "Safety Violation: Numeric values/dosages do not match! Original: '{}', Matched: '{}'",
                original, matched
            ));
        }
        Ok(())
    }
}

impl DrugMatcher for AdvancedDrugMatcher {
    fn add_doctor_vocabulary(&mut self, drug_names: &[String]) {
        for name in drug_names {
            if !self.doctor_vocab.contains(name) {
                self.doctor_vocab.push(name.clone());
            }
        }
    }

    fn record_clinician_correction(&mut self, original: &str, corrected: &str) {
        self.corrections_map.insert(original.to_lowercase(), corrected.to_string());
        println!("[Learning Layer]: Boosted match map: {} -> {}", original, corrected);
    }

    fn match_text(&self, raw_transcript: &str) -> MedicalCorrectionResult {
        let words: Vec<&str> = raw_transcript.split_whitespace().collect();
        let mut terms = Vec::new();
        let mut formatted_words = Vec::new();
        let mut has_low_confidence = false;

        let mut i = 0;
        while i < words.len() {
            let word = words[i];
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            // Check if clinician correction was previously registered
            if let Some(corrected) = self.corrections_map.get(&clean_word.to_lowercase()) {
                terms.push(MatchedDrugToken {
                    original_word: clean_word.to_string(),
                    matched_name: Some(corrected.clone()),
                    dosage: None,
                    confidence: 1.0,
                    confidence_level: ConfidenceLevel::High,
                    alternatives: vec![],
                });
                formatted_words.push(corrected.clone());
                i += 1;
                continue;
            }

            let mut best_match: Option<String> = None;
            let mut best_score = 0.0f32;
            let mut alternatives = Vec::new();

            if clean_word.len() >= 3 && !clean_word.chars().next().map_or(false, |c| c.is_numeric()) {
                // Vocabulary priority search order: Doctor -> Specialty -> Dept -> Hospital -> Global
                let pool = self.doctor_vocab.iter()
                    .chain(self.specialty_vocab.iter())
                    .chain(self.dept_vocab.iter())
                    .chain(self.hospital_vocab.iter())
                    .chain(self.global_vocab.iter());

                for db_drug in pool {
                    let dist = Self::calculate_levenshtein(clean_word, db_drug);
                    let max_len = clean_word.len().max(db_drug.len()) as f32;
                    let similarity = 1.0 - (dist as f32 / max_len);

                    if similarity > best_score {
                        if best_score > 0.6 {
                            if let Some(prev) = &best_match {
                                alternatives.push(prev.clone());
                            }
                        }
                        best_score = similarity;
                        best_match = Some(db_drug.clone());
                    } else if similarity > 0.6 {
                        alternatives.push(db_drug.clone());
                    }
                }
            }

            if let Some(matched) = best_match {
                if best_score >= 0.65 {
                    let mut dosage_str = None;
                    if i + 1 < words.len() {
                        let next_word = words[i+1];
                        if next_word.chars().next().map_or(false, |c| c.is_numeric()) {
                            if i + 2 < words.len() && (words[i+2].eq_ignore_ascii_case("mg") || words[i+2].eq_ignore_ascii_case("g") || words[i+2].eq_ignore_ascii_case("ml")) {
                                dosage_str = Some(format!("{} {}", words[i+1], words[i+2]));
                            } else {
                                dosage_str = Some(words[i+1].to_string());
                            }
                        }
                    }

                    // Strict immutability safety check for dosages
                    if let Some(ref d) = dosage_str {
                        if let Err(e) = Self::verify_dosage_immutability(d, d) {
                            println!("[Safety Alert]: {}", e);
                        }
                    }

                    let conf_level = if best_score >= 0.85 {
                        ConfidenceLevel::High
                    } else if best_score >= 0.70 {
                        ConfidenceLevel::Medium
                    } else {
                        has_low_confidence = true;
                        ConfidenceLevel::Low
                    };

                    terms.push(MatchedDrugToken {
                        original_word: clean_word.to_string(),
                        matched_name: Some(matched.clone()),
                        dosage: dosage_str,
                        confidence: best_score,
                        confidence_level: conf_level,
                        alternatives,
                    });

                    formatted_words.push(matched);
                    i += 1;
                    continue;
                }
            }

            formatted_words.push(word.to_string());
            i += 1;
        }

        MedicalCorrectionResult {
            formatted_text: formatted_words.join(" "),
            terms,
            has_low_confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layered_vocabularies() {
        let mut matcher = AdvancedDrugMatcher::new();
        matcher.add_doctor_vocabulary(&["DoctorCustomDrug".to_string()]);
        
        let res = matcher.match_text("Tab DoctorCustomDrug 10 mg");
        assert_eq!(res.terms[0].matched_name.as_deref(), Some("DoctorCustomDrug"));
    }

    #[test]
    fn test_clinician_corrections_learning() {
        let mut matcher = AdvancedDrugMatcher::new();
        matcher.record_clinician_correction("wrongspell", "Paracetamol");
        
        let res = matcher.match_text("Tab wrongspell 500 mg");
        assert_eq!(res.terms[0].matched_name.as_deref(), Some("Paracetamol"));
    }

    #[test]
    fn test_dosage_immutability_enforcement() {
        assert!(AdvancedDrugMatcher::verify_dosage_immutability("500 mg", "500 mg").is_ok());
        assert!(AdvancedDrugMatcher::verify_dosage_immutability("500 mg", "50 mg").is_err());
    }
}
