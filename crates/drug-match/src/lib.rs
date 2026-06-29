use serde::{Deserialize, Serialize};

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
}

pub struct AdvancedDrugMatcher {
    master_db: Vec<String>,
    doctor_vocab: Vec<String>,
}

impl AdvancedDrugMatcher {
    pub fn new() -> Self {
        let default_drugs = vec![
            "Amlokind".to_string(),
            "Amlodipine".to_string(),
            "Dolo".to_string(),
            "Paracetamol".to_string(),
            "Metformin".to_string(),
            "Glycomet".to_string(),
            "Telmisartan".to_string(),
            "Telma".to_string(),
            "Pantocid".to_string(),
            "Pantoprazole".to_string(),
            "Azithral".to_string(),
            "Azithromycin".to_string(),
            "Augmentin".to_string(),
            "Amoxyclav".to_string(),
            "Ciprofloxacin".to_string(),
            "Ciplox".to_string(),
        ];
        Self {
            master_db: default_drugs,
            doctor_vocab: Vec::new(),
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
}

impl DrugMatcher for AdvancedDrugMatcher {
    fn add_doctor_vocabulary(&mut self, drug_names: &[String]) {
        for name in drug_names {
            if !self.doctor_vocab.contains(name) {
                self.doctor_vocab.push(name.clone());
            }
        }
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
            
            let mut best_match: Option<String> = None;
            let mut best_score = 0.0f32;
            let mut alternatives = Vec::new();

            if clean_word.len() >= 3 && !clean_word.chars().next().map_or(false, |c| c.is_numeric()) {
                let pool = self.doctor_vocab.iter().chain(self.master_db.iter());
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
    super::*;

    #[test]
    fn test_drug_matching_fuzzy() {
        let matcher = AdvancedDrugMatcher::new();
        let res = matcher.match_text("Tab Amlokind 5 mg once daily");
        assert!(res.terms.iter().any(|t| t.matched_name.as_deref() == Some("Amlokind")));
        assert_eq!(res.terms[0].dosage.as_deref(), Some("5 mg"));
    }
}
