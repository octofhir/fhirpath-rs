use std::sync::Arc;

use octofhir_fhirschema::{EmbeddedSchemaProvider, ModelFhirVersion};

#[tokio::main]
async fn main() {
    let provider = Arc::new(EmbeddedSchemaProvider::new(ModelFhirVersion::R5));
    octofhir_fhirpath::lsp::run_stdio(provider).await.unwrap();
}
