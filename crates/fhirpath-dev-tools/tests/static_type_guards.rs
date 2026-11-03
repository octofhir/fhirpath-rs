//! Static analysis uses the declared FHIR name supplied by the embedded schemas.

use std::sync::Arc;

use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};
use octofhir_fhirschema::{EmbeddedSchemaProvider, ModelFhirVersion, ModelProvider};

#[tokio::test]
async fn embedded_models_preserve_focus_types_for_static_analysis() {
    for version in [
        ModelFhirVersion::R4,
        ModelFhirVersion::R4B,
        ModelFhirVersion::R5,
        ModelFhirVersion::R6,
    ] {
        let model = Arc::new(EmbeddedSchemaProvider::new(version.clone()));
        let mut analyzer = StaticAnalyzer::new(model.clone());
        for (type_name, expression, valid) in [
            ("Period", "$this.toString()", false),
            ("Timing", "$this.toString()", false),
            ("dateTime", "$this.toString()", true),
            ("String", "$this.toString()", true),
            ("Quantity", "$this.toString()", true),
            ("Age", "$this.toString()", true),
            (
                "Period",
                "$this is dateTime implies $this.toString() = 'x'",
                true,
            ),
        ] {
            let focus_type = model.get_type(type_name).await.unwrap().unwrap();
            let result = analyzer
                .analyze_expression(expression, AnalysisContext::new(focus_type))
                .await;
            assert_eq!(
                result.success, valid,
                "{version:?} {type_name}: {expression}: {:?}",
                result.diagnostics
            );
        }
    }
}
