#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskAssessment {
    pub requires_confirmation: bool,
    pub matches: Vec<String>,
}

#[must_use]
pub fn assess_prompt(prompt: &str) -> RiskAssessment {
    const SIGNALS: &[&str] = &[
        "rm",
        "sudo",
        "git push",
        "docker prune",
        "terraform destroy",
        "format disk",
        "drop database",
    ];
    let lower = prompt.to_lowercase();
    let matches = SIGNALS
        .iter()
        .filter(|signal| {
            if signal.contains(' ') {
                lower.contains(**signal)
            } else {
                lower
                    .split_whitespace()
                    .map(|word| word.trim_matches(|character: char| !character.is_alphanumeric()))
                    .any(|word| word == **signal)
            }
        })
        .map(|signal| (*signal).to_owned())
        .collect::<Vec<_>>();
    RiskAssessment {
        requires_confirmation: !matches.is_empty(),
        matches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dangerous_prompt_requires_confirmation() {
        let risk = assess_prompt("Please run git push after the tests");
        assert!(risk.requires_confirmation);
        assert_eq!(risk.matches, vec!["git push"]);
        assert!(assess_prompt("rm").requires_confirmation);
    }
}
