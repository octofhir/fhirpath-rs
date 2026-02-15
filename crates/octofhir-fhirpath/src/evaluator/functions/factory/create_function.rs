//! %factory.create(type) function implementation
//!
//! Creates an empty FHIR type instance.
//! Syntax: %factory.create(type)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryCreateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryCreateFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "create".to_string(),
                description: "Creates an empty instance of the specified FHIR type".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "type".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "FHIR type name to create".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

/// Known FHIR resource types that should include resourceType in their JSON
const RESOURCE_TYPES: &[&str] = &[
    "Account",
    "ActivityDefinition",
    "AdverseEvent",
    "AllergyIntolerance",
    "Appointment",
    "AppointmentResponse",
    "AuditEvent",
    "Basic",
    "Binary",
    "BiologicallyDerivedProduct",
    "BodyStructure",
    "Bundle",
    "CapabilityStatement",
    "CarePlan",
    "CareTeam",
    "CatalogEntry",
    "ChargeItem",
    "ChargeItemDefinition",
    "Claim",
    "ClaimResponse",
    "ClinicalImpression",
    "CodeSystem",
    "Communication",
    "CommunicationRequest",
    "CompartmentDefinition",
    "Composition",
    "ConceptMap",
    "Condition",
    "Consent",
    "Contract",
    "Coverage",
    "CoverageEligibilityRequest",
    "CoverageEligibilityResponse",
    "DetectedIssue",
    "Device",
    "DeviceDefinition",
    "DeviceMetric",
    "DeviceRequest",
    "DeviceUseStatement",
    "DiagnosticReport",
    "DocumentManifest",
    "DocumentReference",
    "EffectEvidenceSynthesis",
    "Encounter",
    "Endpoint",
    "EnrollmentRequest",
    "EnrollmentResponse",
    "EpisodeOfCare",
    "EventDefinition",
    "Evidence",
    "EvidenceVariable",
    "ExampleScenario",
    "ExplanationOfBenefit",
    "FamilyMemberHistory",
    "Flag",
    "Goal",
    "GraphDefinition",
    "Group",
    "GuidanceResponse",
    "HealthcareService",
    "ImagingStudy",
    "Immunization",
    "ImmunizationEvaluation",
    "ImmunizationRecommendation",
    "ImplementationGuide",
    "InsurancePlan",
    "Invoice",
    "Library",
    "Linkage",
    "List",
    "Location",
    "Measure",
    "MeasureReport",
    "Media",
    "Medication",
    "MedicationAdministration",
    "MedicationDispense",
    "MedicationKnowledge",
    "MedicationRequest",
    "MedicationStatement",
    "MedicinalProduct",
    "MessageDefinition",
    "MessageHeader",
    "MolecularSequence",
    "NamingSystem",
    "NutritionOrder",
    "Observation",
    "ObservationDefinition",
    "OperationDefinition",
    "OperationOutcome",
    "Organization",
    "OrganizationAffiliation",
    "Parameters",
    "Patient",
    "PaymentNotice",
    "PaymentReconciliation",
    "Person",
    "PlanDefinition",
    "Practitioner",
    "PractitionerRole",
    "Procedure",
    "Provenance",
    "Questionnaire",
    "QuestionnaireResponse",
    "RelatedPerson",
    "RequestGroup",
    "ResearchDefinition",
    "ResearchElementDefinition",
    "ResearchStudy",
    "ResearchSubject",
    "RiskAssessment",
    "RiskEvidenceSynthesis",
    "Schedule",
    "SearchParameter",
    "ServiceRequest",
    "Slot",
    "Specimen",
    "SpecimenDefinition",
    "StructureDefinition",
    "StructureMap",
    "Subscription",
    "Substance",
    "SubstanceNucleicAcid",
    "SubstancePolymer",
    "SubstanceProtein",
    "SubstanceReferenceInformation",
    "SubstanceSourceMaterial",
    "SubstanceSpecification",
    "SupplyDelivery",
    "SupplyRequest",
    "Task",
    "TerminologyCapabilities",
    "TestReport",
    "TestScript",
    "ValueSet",
    "VerificationResult",
    "VisionPrescription",
];

fn is_resource_type(type_name: &str) -> bool {
    RESOURCE_TYPES.contains(&type_name)
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryCreateFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        // Check if called on %factory variable OR as a standalone factory function
        let is_factory = input.len() == 1 && is_factory_variable(&input[0]);
        if !is_factory {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "create function can only be called on %factory variable".to_string(),
            ));
        }

        let type_name = match args.first().and_then(|a| a.first()) {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "create function requires a string type argument".to_string(),
                ));
            }
        };

        // Strip FHIR. prefix if present
        let clean_type = type_name.strip_prefix("FHIR.").unwrap_or(&type_name);

        let mut obj = serde_json::Map::new();

        // If it's a resource type, add resourceType field
        if is_resource_type(clean_type) {
            obj.insert(
                "resourceType".to_string(),
                serde_json::Value::String(clean_type.to_string()),
            );
        }

        let type_info = TypeInfo::new_complex(clean_type);
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(obj),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
