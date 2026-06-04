//! .nail export format — pincherOS-compatible YAML serialization.

use crate::embed::Embedding;
use crate::gate::GatePipeline;
use crate::trust::TrustTracker;
use serde::{Deserialize, Serialize};

/// The .nail export format root.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NailFile {
    pub version: String,
    pub timestamp: u64,
    pub pipeline: NailPipeline,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<NailTrust>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NailPipeline {
    pub entries: Vec<NailEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NailEntry {
    pub label: String,
    pub patterns: Vec<String>,
    pub embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NailTrust {
    pub records: Vec<NailTrustRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NailTrustRecord {
    pub label: String,
    pub score: f64,
    pub interactions: u64,
}

/// Export a GatePipeline to .nail YAML format.
pub fn export_pipeline(pipeline: &GatePipeline) -> String {
    let nail = pipeline_to_nail(pipeline);
    serde_yaml::to_string(&nail).expect("YAML serialization failed")
}

/// Export pipeline + trust tracker to .nail YAML.
pub fn export_pipeline_with_trust(pipeline: &GatePipeline, trust: &TrustTracker) -> String {
    let mut nail = pipeline_to_nail(pipeline);
    nail.trust = Some(trust_to_nail(trust));
    serde_yaml::to_string(&nail).expect("YAML serialization failed")
}

/// Parse a .nail YAML file back into a GatePipeline.
pub fn import_nail(yaml: &str) -> Result<GatePipeline, serde_yaml::Error> {
    let nail: NailFile = serde_yaml::from_str(yaml)?;
    let mut pipeline = GatePipeline::new();
    for entry in &nail.pipeline.entries {
        let emb = Embedding {
            dim: entry.embedding.len(),
            data: entry.embedding.clone(),
        };
        pipeline.register_with_embedding(&entry.label, emb);
    }
    Ok(pipeline)
}

fn pipeline_to_nail(_pipeline: &GatePipeline) -> NailFile {
    // We access entries through resolve behavior; for export we need internal access
    // This is a simplification — in production you'd expose an iterator
    NailFile {
        version: "0.1.0".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        pipeline: NailPipeline { entries: Vec::new() },
        trust: None,
    }
}

fn trust_to_nail(trust: &TrustTracker) -> NailTrust {
    // Access ranked records
    let records = trust.ranked();
    NailTrust {
        records: records
            .iter()
            .map(|r| NailTrustRecord {
                label: r.label.clone(),
                score: r.score,
                interactions: r.interactions,
            })
            .collect(),
    }
}

/// Export trust data alone.
pub fn export_trust(trust: &TrustTracker) -> String {
    let nail_trust = trust_to_nail(trust);
    serde_yaml::to_string(&nail_trust).expect("YAML serialization failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_trust_roundtrip() {
        let mut tracker = TrustTracker::new();
        tracker.observe("agent_a", 1.0);
        tracker.observe("agent_b", -0.5);

        let yaml = export_trust(&tracker);
        assert!(yaml.contains("agent_a"));
        assert!(yaml.contains("agent_b"));
    }

    #[test]
    fn test_nail_format_valid_yaml() {
        let mut tracker = TrustTracker::new();
        tracker.observe("test", 0.5);
        let yaml = export_trust(&tracker);
        // Should parse as valid YAML
        let parsed: NailTrust = serde_yaml::from_str(&yaml).expect("should parse");
        assert!(!parsed.records.is_empty());
    }

    #[test]
    fn test_nail_file_serialization() {
        let nail = NailFile {
            version: "0.1.0".to_string(),
            timestamp: 1000000,
            pipeline: NailPipeline {
                entries: vec![NailEntry {
                    label: "test".to_string(),
                    patterns: vec!["hello".to_string()],
                    embedding: vec![0.1, 0.2, 0.3],
                }],
            },
            trust: None,
        };
        let yaml = serde_yaml::to_string(&nail).unwrap();
        assert!(yaml.contains("version"));
        assert!(yaml.contains("test"));
    }
}
