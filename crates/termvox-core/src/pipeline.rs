use std::collections::BTreeMap;

use crate::PipelineConfig;

pub trait PromptStage: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, input: &str) -> String;
}

pub struct PromptPipeline {
    stages: Vec<Box<dyn PromptStage>>,
}

impl PromptPipeline {
    #[must_use]
    pub fn from_config(config: &PipelineConfig) -> Self {
        let mut stages: Vec<Box<dyn PromptStage>> = vec![Box::new(WhitespaceStage)];
        if !config.dictionary.is_empty() {
            stages.push(Box::new(DictionaryStage(config.dictionary.clone())));
        }
        if config.prefix.is_some() || config.suffix.is_some() {
            stages.push(Box::new(ProfileStage {
                prefix: config.prefix.clone(),
                suffix: config.suffix.clone(),
            }));
        }
        Self { stages }
    }

    #[must_use]
    pub fn process(&self, input: &str) -> String {
        self.stages
            .iter()
            .fold(input.to_owned(), |text, stage| stage.apply(&text))
    }
}

struct WhitespaceStage;

impl PromptStage for WhitespaceStage {
    fn name(&self) -> &'static str {
        "whitespace"
    }

    fn apply(&self, input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

struct DictionaryStage(BTreeMap<String, String>);

impl PromptStage for DictionaryStage {
    fn name(&self) -> &'static str {
        "dictionary"
    }

    fn apply(&self, input: &str) -> String {
        self.0
            .iter()
            .fold(input.to_owned(), |text, (from, to)| text.replace(from, to))
    }
}

struct ProfileStage {
    prefix: Option<String>,
    suffix: Option<String>,
}

impl PromptStage for ProfileStage {
    fn name(&self) -> &'static str {
        "profile"
    }

    fn apply(&self, input: &str) -> String {
        [
            self.prefix.as_deref().unwrap_or_default(),
            input,
            self.suffix.as_deref().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_is_conservative_and_applies_explicit_profile() {
        let config = PipelineConfig {
            dictionary: BTreeMap::from([("api rest".into(), "REST API".into())]),
            prefix: Some("Project context".into()),
            suffix: None,
        };
        let output = PromptPipeline::from_config(&config).process("  crea   api rest ");
        assert_eq!(output, "Project context\n\ncrea REST API");
    }
}
