//! Benchmark the FHIR R4 `dom-3` invariant against a bundle file.
//!
//! Usage: cargo run --release -p fhirpath-dev-tools --example dom3-bench -- <bundle.json> [runs]

use octofhir_fhir_model::FhirVersion;
use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, FhirPathValue};
use octofhir_fhirschema::EmbeddedSchemaProvider;
use std::sync::Arc;
use std::time::Instant;

const DOM3: &str = "contained.where((('#'+id in (%resource.descendants().reference | %resource.descendants().as(canonical) | %resource.descendants().as(uri) | %resource.descendants().as(url))) or descendants().where(reference = '#').exists() or descendants().where(as(canonical) = '#').exists() or descendants().where(as(uri) = '#').exists()).not()).trace('unmatched', id).empty()";

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        anyhow::bail!(
            "missing bundle path\nusage: dom3-bench <bundle.json> [runs] [limit]  (env: SKIP_BUNDLE=1)"
        );
    };
    let runs: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);
    let limit: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let skip_bundle = std::env::var("SKIP_BUNDLE").is_ok();

    let t0 = Instant::now();
    let raw = std::fs::read_to_string(&path)?;
    let bundle: serde_json::Value = serde_json::from_str(&raw)?;
    println!(
        "parse json: {:?} ({:.1} MB)",
        t0.elapsed(),
        raw.len() as f64 / 1_048_576.0
    );

    let t0 = Instant::now();
    let registry = Arc::new(octofhir_fhirpath::create_function_registry());
    let model_provider = Arc::new(EmbeddedSchemaProvider::new(FhirVersion::R4))
        as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>;
    let engine = FhirPathEngine::new(registry, model_provider.clone()).await?;
    println!("engine init: {:?}", t0.elapsed());

    let all_resources: Vec<serde_json::Value> = bundle
        .get("entry")
        .and_then(|e| e.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| e.get("resource").cloned())
                .collect()
        })
        .unwrap_or_default();
    let resources: Vec<serde_json::Value> =
        all_resources.iter().take(limit).cloned().collect();
    let contained_total: usize = resources
        .iter()
        .map(|r| {
            r.get("contained")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0)
        })
        .sum();
    println!(
        "entries: {}, contained resources total: {}",
        resources.len(),
        contained_total
    );

    // ---- breakdown of dom-3 into its component subexpressions, on the worst resource ----
    if std::env::var("PARTS").is_ok() {
        let worst = resources
            .iter()
            .max_by_key(|r| {
                serde_json::to_string(r).map(|s| s.len()).unwrap_or(0)
                    * r.get("contained")
                        .and_then(|c| c.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
            })
            .expect("bundle should have at least one resource");
        println!("\nbreakdown on worst resource:");
        for expr in [
            "contained.count()",
            "contained.where(true).count()",
            "contained.where(id.exists()).count()",
            "contained.where('#'+id = 'zzz').count()",
            "%resource.descendants().count()",
            "%resource.descendants().reference.count()",
            "%resource.descendants().as(canonical).count()",
            "%resource.descendants().as(uri).count()",
            "(%resource.descendants().reference | %resource.descendants().as(canonical) | %resource.descendants().as(uri) | %resource.descendants().as(url)).count()",
            "contained.where(('#'+id in (%resource.descendants().reference | %resource.descendants().as(canonical) | %resource.descendants().as(uri) | %resource.descendants().as(url))).not()).count()",
            "contained.where(descendants().where(reference = '#').exists().not()).count()",
            DOM3,
        ] {
            let collection = Collection::single(FhirPathValue::resource(worst.clone()));
            let ctx = EvaluationContext::new(collection, model_provider.clone(), None, None, None);
            let t = Instant::now();
            let r = engine.evaluate(expr, &ctx).await;
            let label: String = expr.chars().take(72).collect();
            println!(
                "  {:>9.1} ms  {} -> {}",
                t.elapsed().as_secs_f64() * 1000.0,
                label,
                match &r {
                    Ok(v) => format!("{:?}", v.value.first()),
                    Err(e) => format!("ERROR: {e}"),
                }
            );
        }
    }

    // ---- per-resource evaluation (real validator semantics: %resource = the resource) ----
    for run in 1..=runs {
        let mut per_resource: Vec<f64> = Vec::with_capacity(resources.len());
        let mut failures = 0usize;
        let mut errors = 0usize;
        let start = Instant::now();
        for (i, res) in resources.iter().enumerate() {
            let size = serde_json::to_string(res).map(|s| s.len()).unwrap_or(0);
            let ncontained = res
                .get("contained")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let collection = Collection::single(FhirPathValue::resource(res.clone()));
            let ctx = EvaluationContext::new(collection, model_provider.clone(), None, None, None);
            let t = Instant::now();
            let result = engine.evaluate(DOM3, &ctx).await;
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            println!(
                "  [{i}] {} bytes={} contained={} -> {:.1} ms",
                res.get("resourceType").and_then(|v| v.as_str()).unwrap_or("?"),
                size,
                ncontained,
                ms
            );
            use std::io::Write;
            let _ = std::io::stdout().flush();
            per_resource.push(ms);
            match result {
                Ok(r) => {
                    if !matches!(r.value.first(), Some(FhirPathValue::Boolean(true, _, _))) {
                        failures += 1;
                    }
                }
                Err(_) => errors += 1,
            }
        }
        let total = start.elapsed();
        per_resource.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sum: f64 = per_resource.iter().sum();
        println!(
            "\nrun {run}: per-resource dom-3 over {} resources\n  total    {:?}\n  mean     {:.3} ms\n  p50      {:.3} ms\n  p95      {:.3} ms\n  max      {:.3} ms\n  min      {:.3} ms\n  throughput {:.1} res/sec\n  constraint-false: {failures}, errors: {errors}",
            resources.len(),
            total,
            sum / per_resource.len() as f64,
            pct(&per_resource, 0.50),
            pct(&per_resource, 0.95),
            per_resource[per_resource.len() - 1],
            per_resource[0],
            resources.len() as f64 / total.as_secs_f64(),
        );
    }

    // ---- whole-bundle evaluation (%resource = Bundle) ----
    
    for run in 1..=runs {
        if skip_bundle {
            break;
        }
        let collection = Collection::single(FhirPathValue::resource(bundle.clone()));
        let ctx = EvaluationContext::new(collection, model_provider.clone(), None, None, None);
        let t = Instant::now();
        let r = engine.evaluate(DOM3, &ctx).await;
        println!(
            "whole-bundle run {run}: {:?} -> {}",
            t.elapsed(),
            match &r {
                Ok(v) => format!("{:?}", v.value.first()),
                Err(e) => format!("ERROR: {e}"),
            }
        );
    }

    Ok(())
}
