use crate::{get_sample_bundle, get_sample_patient};
use anyhow::Result;
use octofhir_fhirpath_evaluator::FhirPathEngine;
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use octofhir_fhirpath_parser::{Tokenizer, parse_expression};
use octofhir_fhirpath_registry::FhirPathRegistry;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ProfileRunner {
    output_dir: PathBuf,
    iterations: usize,
    use_bundle: bool,
}

impl ProfileRunner {
    pub fn new(output_dir: PathBuf, iterations: usize, use_bundle: bool) -> Self {
        Self {
            output_dir,
            iterations,
            use_bundle,
        }
    }

    pub async fn profile_expression(&self, expression: &str) -> Result<()> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Generate a safe filename from the expression
        let safe_name = expression
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let _flamegraph_path = self.output_dir.join(format!("{safe_name}.svg"));
        let _profile_path = self.output_dir.join(format!("{safe_name}.profile"));
        let _summary_path = self.output_dir.join(format!("{safe_name}_summary.txt"));

        println!("Starting profiling for: {expression}");
        println!("Iterations: {}", self.iterations);

        // Profile tokenization
        self.profile_tokenization(expression, &safe_name).await?;

        // Profile parsing
        self.profile_parsing(expression, &safe_name).await?;

        // Profile evaluation
        self.profile_evaluation(expression, &safe_name).await?;

        println!("Profiling completed!");
        println!("Check output directory: {}", self.output_dir.display());

        Ok(())
    }

    async fn profile_tokenization(&self, expression: &str, safe_name: &str) -> Result<()> {
        let output_path = self.output_dir.join(format!("{safe_name}_tokenize.svg"));

        println!("Profiling tokenization...");
        let guard = pprof::ProfilerGuard::new(100)?;

        for _ in 0..self.iterations {
            let _ = Tokenizer::new(expression).tokenize_all();
        }

        let report = guard.report().build()?;
        self.generate_flamegraph_from_report(report, output_path, "Tokenization")?;

        Ok(())
    }

    async fn profile_parsing(&self, expression: &str, safe_name: &str) -> Result<()> {
        let output_path = self.output_dir.join(format!("{safe_name}_parse.svg"));

        println!("Profiling parsing...");
        let guard = pprof::ProfilerGuard::new(100)?;

        for _ in 0..self.iterations {
            let _ = parse_expression(expression);
        }

        let report = guard.report().build()?;
        self.generate_flamegraph_from_report(report, output_path, "Parsing")?;

        Ok(())
    }

    async fn profile_evaluation(&self, expression: &str, safe_name: &str) -> Result<()> {
        let output_path = self.output_dir.join(format!("{safe_name}_evaluate.svg"));

        println!("Profiling evaluation...");
        let guard = pprof::ProfilerGuard::new(100)?;

        let registry = Arc::new(FhirPathRegistry::default());
        let model_provider = Arc::new(
            FhirSchemaModelProvider::r5()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create R5 FHIR Schema Provider: {}", e))?,
        );
        let engine = FhirPathEngine::new(registry, model_provider);
        let data = if self.use_bundle {
            get_sample_bundle()
        } else {
            get_sample_patient()
        };

        for _ in 0..self.iterations {
            let _ = engine.evaluate(expression, data.clone()).await;
        }

        let report = guard.report().build()?;
        self.generate_flamegraph_from_report(report, output_path, "Evaluation")?;

        Ok(())
    }

    fn generate_flamegraph_from_report(
        &self,
        report: pprof::Report,
        output_path: PathBuf,
        operation: &str,
    ) -> Result<()> {
        println!("Generating flamegraph for {operation} operation...");

        // Use pprof's built-in flamegraph generation
        let file = std::fs::File::create(&output_path)?;
        report.flamegraph(file)?;

        println!("Flamegraph generated: {}", output_path.display());

        Ok(())
    }

    pub fn generate_summary(&self, expression: &str) -> String {
        format!(
            r#"# Profiling Summary

Expression: {}
Iterations: {}
Data Source: {}
Output Directory: {}

## Files Generated

1. `{}_tokenize.svg` - Tokenization flamegraph
2. `{}_parse.svg` - Parsing flamegraph
3. `{}_evaluate.svg` - Evaluation flamegraph

## Usage

To view flamegraphs, open the SVG files in a web browser.

## Notes

- Profiling was performed with {} iterations
- Data source: {}
- All times are in CPU time, not wall-clock time
- Flamegraphs show function call stacks and their relative CPU usage

## Next Steps

1. Review flamegraphs to identify performance bottlenecks
2. Focus optimization efforts on the hottest code paths
3. Compare results across different expression complexities
"#,
            expression,
            self.iterations,
            if self.use_bundle { "Bundle" } else { "Patient" },
            self.output_dir.display(),
            expression
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                })
                .collect::<String>()
                .chars()
                .take(50)
                .collect::<String>(),
            expression
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                })
                .collect::<String>()
                .chars()
                .take(50)
                .collect::<String>(),
            expression
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                })
                .collect::<String>()
                .chars()
                .take(50)
                .collect::<String>(),
            self.iterations,
            if self.use_bundle {
                "bundle-medium.json"
            } else {
                "sample patient"
            },
        )
    }
}
