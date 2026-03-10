/// Rule defines a keyword-matching rule for a client matter.
pub struct Rule {
    pub matter_id: i64,
    pub keywords: Vec<String>, // loaded from matters.keywords JSON column
    pub weight: f32,
}

/// Classify a window title against a set of rules.
/// Returns the `matter_id` of the best-matching rule, or `None` if no match.
/// Performance target: < 1ms per call. Runs entirely locally.
pub fn classify(title: &str, rules: &[Rule]) -> Option<i64> {
    let title_lower = title.to_lowercase();
    let mut best: Option<(i64, f32)> = None;
    for rule in rules {
        let score: f32 = rule
            .keywords
            .iter()
            .filter(|kw| title_lower.contains(&kw.to_lowercase()))
            .map(|kw| rule.weight * kw.len() as f32) // longer keyword = higher confidence
            .sum();
        if score > 0.0 && best.map_or(true, |(_, s)| score > s) {
            best = Some((rule.matter_id, score));
        }
    }
    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_single_keyword_match() {
        let rules = vec![Rule {
            matter_id: 1,
            keywords: vec!["Smith".to_string()],
            weight: 1.0,
        }];
        let result = classify("Smith v Doe - Document Draft.docx", &rules);
        assert_eq!(result, Some(1));
        println!("✅ Single keyword match: 'Smith' matched matter_id=1");
    }

    #[test]
    fn test_classify_no_match() {
        let rules = vec![Rule {
            matter_id: 1,
            keywords: vec!["Smith".to_string()],
            weight: 1.0,
        }];
        let result = classify("Budget Spreadsheet Q4.xlsx", &rules);
        assert_eq!(result, None);
        println!("✅ No match: unrelated title returns None");
    }

    #[test]
    fn test_classify_best_match_wins() {
        let rules = vec![
            Rule {
                matter_id: 1,
                keywords: vec!["Smith".to_string()],
                weight: 1.0,
            },
            Rule {
                matter_id: 2,
                keywords: vec!["Smith v Doe".to_string(), "Case 990".to_string()],
                weight: 1.0,
            },
        ];
        // "Smith v Doe" is a longer keyword match, so matter_id=2 should win
        let result = classify("Smith v Doe - Motion to Dismiss.pdf", &rules);
        assert_eq!(result, Some(2));
        println!("✅ Best match wins: longer keyword 'Smith v Doe' beats 'Smith'");
    }

    #[test]
    fn test_classify_case_insensitive() {
        let rules = vec![Rule {
            matter_id: 3,
            keywords: vec!["ACME Corp".to_string()],
            weight: 1.0,
        }];
        let result = classify("acme corp - invoice review", &rules);
        assert_eq!(result, Some(3));
        println!("✅ Case insensitive: 'acme corp' matched 'ACME Corp'");
    }

    #[test]
    fn test_classify_performance() {
        // Simulate 100 rules with 5 keywords each
        let rules: Vec<Rule> = (0..100)
            .map(|i| Rule {
                matter_id: i,
                keywords: (0..5).map(|j| format!("keyword_{}_{}", i, j)).collect(),
                weight: 1.0,
            })
            .collect();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            classify("keyword_50_3 - Some Document.docx", &rules);
        }
        let elapsed = start.elapsed();
        let per_call = elapsed / 1000;
        println!("✅ Performance: 1000 calls in {:?} ({:?}/call)", elapsed, per_call);
        assert!(
            per_call.as_millis() < 1,
            "classify() should complete in < 1ms, took {:?}",
            per_call
        );
    }
}
