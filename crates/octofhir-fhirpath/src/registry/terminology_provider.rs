//! Enhanced TerminologyProvider trait and DefaultTerminologyProvider implementation
//!
//! This module provides a more sophisticated terminology provider interface that extends
//! beyond the basic TerminologyService to provide comprehensive FHIR terminology operations
//! with integration to external terminology servers like tx.fhir.org.

use async_trait::async_trait;
use serde_json::{Value, json};

use super::terminology_utils::{
    Coding, ConceptDesignation, ConceptProperty, ConceptTranslation,
    TerminologyUtils,
};
use crate::core::error_code::FP0200;
use crate::core::{FhirPathError, FhirPathValue, Result};

/// Comprehensive details about a concept returned by terminology operations
#[derive(Debug, Clone)]
pub struct ConceptDetails {
    pub name: Option<String>,
    pub version: Option<String>,
    pub display: String,
    pub designation: Vec<ConceptDesignation>,
    pub property: Vec<ConceptProperty>,
}

/// Enhanced TerminologyProvider trait that provides comprehensive FHIR terminology operations
/// This extends beyond the basic TerminologyService to provide detailed concept information,
/// designations, properties, and advanced validation capabilities.
#[async_trait]
pub trait TerminologyProvider: Send + Sync {
    /// Check if a coding is a member of a ValueSet
    ///
    /// # Arguments
    /// * `coding` - The coding to check
    /// * `valueset_url` - URL or identifier of the ValueSet
    ///
    /// # Returns
    /// * `Result<bool>` - True if the coding is a member of the ValueSet
    async fn check_valueset_membership(&self, coding: &Coding, valueset_url: &str) -> Result<bool>;

    /// Translate a concept using a ConceptMap
    ///
    /// # Arguments
    /// * `coding` - The coding to translate
    /// * `conceptmap_url` - URL or identifier of the ConceptMap
    /// * `reverse` - Whether to translate in reverse direction
    ///
    /// # Returns
    /// * `Result<Vec<FhirPathValue>>` - Collection of translation results
    async fn translate_concept(
        &self,
        coding: &Coding,
        conceptmap_url: &str,
        reverse: bool,
    ) -> Result<Vec<FhirPathValue>>;

    /// Validate that a code is valid in a code system
    ///
    /// # Arguments
    /// * `system` - Code system URL
    /// * `code` - Code to validate
    ///
    /// # Returns
    /// * `Result<bool>` - True if the code is valid in the system
    async fn validate_code(&self, system: &str, code: &str) -> Result<bool>;

    /// Check if one code subsumes another (hierarchical relationship)
    ///
    /// # Arguments
    /// * `coding_a` - The potentially subsumming code
    /// * `coding_b` - The potentially subsumed code
    ///
    /// # Returns
    /// * `Result<bool>` - True if coding_a subsumes coding_b
    async fn check_subsumption(&self, coding_a: &Coding, coding_b: &Coding) -> Result<bool>;

    /// Get designations for a coding in specified language/use
    ///
    /// # Arguments
    /// * `coding` - The coding to get designations for
    /// * `language` - Optional language code (e.g., 'en', 'es')
    /// * `use_code` - Optional designation use code
    ///
    /// # Returns
    /// * `Result<Vec<FhirPathValue>>` - Collection of designation objects
    async fn get_designations(
        &self,
        coding: &Coding,
        language: Option<&str>,
        use_code: Option<&str>,
    ) -> Result<Vec<FhirPathValue>>;

    /// Get properties for a concept
    ///
    /// # Arguments
    /// * `coding` - The coding to get properties for
    /// * `property` - Property name to retrieve
    ///
    /// # Returns
    /// * `Result<Vec<FhirPathValue>>` - Collection of property values
    async fn get_concept_properties(
        &self,
        coding: &Coding,
        property: &str,
    ) -> Result<Vec<FhirPathValue>>;

    /// Get the terminology server base URL
    async fn get_terminology_server_url(&self) -> Result<String>;

    /// Expand a ValueSet to get all member codes
    ///
    /// # Arguments
    /// * `valueset_url` - URL or identifier of the ValueSet to expand
    ///
    /// # Returns
    /// * `Result<Vec<Coding>>` - Collection of all codes in the ValueSet
    async fn expand_valueset(&self, valueset_url: &str) -> Result<Vec<Coding>>;

    /// Lookup comprehensive concept details
    ///
    /// # Arguments
    /// * `coding` - The coding to look up
    ///
    /// # Returns
    /// * `Result<Option<ConceptDetails>>` - Detailed concept information, or None if not found
    async fn lookup_concept(&self, coding: &Coding) -> Result<Option<ConceptDetails>>;
}

/// Default terminology provider that integrates with tx.fhir.org
///
/// This provider implements the FHIR Terminology Service specification using
/// the public tx.fhir.org terminology server. It supports all standard operations
/// including ValueSet validation, ConceptMap translation, code system validation,
/// subsumption checking, designation lookup, and property retrieval.
#[derive(Debug)]
pub struct DefaultTerminologyProvider {
    server_url: String,
    client: reqwest::Client,
}

impl DefaultTerminologyProvider {
    /// Create a new DefaultTerminologyProvider using tx.fhir.org
    pub fn new() -> Self {
        Self::with_server_url("https://tx.fhir.org/r4")
    }

    /// Create a new DefaultTerminologyProvider with custom server URL
    ///
    /// # Arguments
    /// * `server_url` - Base URL of the FHIR terminology server
    pub fn with_server_url(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Helper method to parse translation matches from FHIR Parameters response
    fn parse_translation_match(&self, match_parts: &[Value]) -> Result<ConceptTranslation> {
        let mut equivalence = String::new();
        let mut system = String::new();
        let mut code = String::new();
        let mut display = None;
        let mut comment = None;

        for part in match_parts {
            if let Some(part_obj) = part.as_object() {
                if let Some(name) = part_obj.get("name").and_then(|v| v.as_str()) {
                    match name {
                        "equivalence" => {
                            if let Some(eq) = part_obj.get("valueCode").and_then(|v| v.as_str()) {
                                equivalence = eq.to_string();
                            }
                        }
                        "concept" => {
                            if let Some(concept) = part_obj.get("valueCoding") {
                                if let Some(sys) = concept.get("system").and_then(|v| v.as_str()) {
                                    system = sys.to_string();
                                }
                                if let Some(cd) = concept.get("code").and_then(|v| v.as_str()) {
                                    code = cd.to_string();
                                }
                                if let Some(disp) = concept.get("display").and_then(|v| v.as_str())
                                {
                                    display = Some(disp.to_string());
                                }
                            }
                        }
                        "comment" => {
                            if let Some(comm) = part_obj.get("valueString").and_then(|v| v.as_str())
                            {
                                comment = Some(comm.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if system.is_empty() || code.is_empty() || equivalence.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0200,
                "Invalid translation match format".to_string(),
            ));
        }

        let mut coding = Coding::new(system, code);
        if let Some(disp) = display {
            coding = coding.with_display(disp);
        }

        Ok(ConceptTranslation {
            equivalence,
            concept: coding,
            comment,
        })
    }

    /// Helper method to parse designation from FHIR Parameters response
    fn parse_designation_part(&self, designation_parts: &[Value]) -> Result<ConceptDesignation> {
        let mut language = None;
        let mut use_coding = None;
        let mut value = String::new();

        for part in designation_parts {
            if let Some(part_obj) = part.as_object() {
                if let Some(name) = part_obj.get("name").and_then(|v| v.as_str()) {
                    match name {
                        "language" => {
                            if let Some(lang) = part_obj.get("valueCode").and_then(|v| v.as_str()) {
                                language = Some(lang.to_string());
                            }
                        }
                        "use" => {
                            if let Some(use_val) = part_obj.get("valueCoding") {
                                if let Ok(coding_val) =
                                    TerminologyUtils::extract_coding_from_json(use_val)
                                {
                                    use_coding = Some(coding_val);
                                }
                            }
                        }
                        "value" => {
                            if let Some(val) = part_obj.get("valueString").and_then(|v| v.as_str())
                            {
                                value = val.to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if value.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0200,
                "Invalid designation format".to_string(),
            ));
        }

        Ok(ConceptDesignation {
            language,
            use_coding,
            value,
        })
    }

    /// Helper method to parse property values from FHIR Parameters response
    fn parse_property_part(
        &self,
        property_parts: &[Value],
        property_name: &str,
    ) -> Result<FhirPathValue> {
        for part in property_parts {
            if let Some(part_obj) = part.as_object() {
                if let Some(name) = part_obj.get("name").and_then(|v| v.as_str()) {
                    if name == "code" {
                        if let Some(code) = part_obj.get("valueCode").and_then(|v| v.as_str()) {
                            if code == property_name {
                                // Look for value in remaining parts
                                for value_part in property_parts {
                                    if let Some(value_obj) = value_part.as_object() {
                                        if let Some(value_name) =
                                            value_obj.get("name").and_then(|v| v.as_str())
                                        {
                                            match value_name {
                                                "valueString" => {
                                                    if let Some(s) = value_obj
                                                        .get("valueString")
                                                        .and_then(|v| v.as_str())
                                                    {
                                                        return Ok(FhirPathValue::String(
                                                            s.to_string(),
                                                        ));
                                                    }
                                                }
                                                "valueBoolean" => {
                                                    if let Some(b) = value_obj
                                                        .get("valueBoolean")
                                                        .and_then(|v| v.as_bool())
                                                    {
                                                        return Ok(FhirPathValue::Boolean(b));
                                                    }
                                                }
                                                "valueInteger" => {
                                                    if let Some(i) = value_obj
                                                        .get("valueInteger")
                                                        .and_then(|v| v.as_i64())
                                                    {
                                                        return Ok(FhirPathValue::Integer(i));
                                                    }
                                                }
                                                "valueCoding" => {
                                                    if let Some(coding) =
                                                        value_obj.get("valueCoding")
                                                    {
                                                        if let Ok(coding_val) = TerminologyUtils::extract_coding_from_json(coding) {
                                                            return Ok(TerminologyUtils::coding_to_value(&coding_val));
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        Err(FhirPathError::evaluation_error(
            FP0200,
            format!("Property '{}' not found", property_name),
        ))
    }

    /// Helper method to parse concept details from FHIR Parameters response
    fn parse_concept_details(&self, parameters: &Value) -> Result<ConceptDetails> {
        let mut name = None;
        let mut version = None;
        let mut display = String::new();
        let mut designation = Vec::new();
        let property = Vec::new();

        if let Some(parameter_array) = parameters.get("parameter").and_then(|v| v.as_array()) {
            for param in parameter_array {
                if let Some(param_obj) = param.as_object() {
                    if let Some(param_name) = param_obj.get("name").and_then(|v| v.as_str()) {
                        match param_name {
                            "name" => {
                                if let Some(n) =
                                    param_obj.get("valueString").and_then(|v| v.as_str())
                                {
                                    name = Some(n.to_string());
                                }
                            }
                            "version" => {
                                if let Some(v) =
                                    param_obj.get("valueString").and_then(|v| v.as_str())
                                {
                                    version = Some(v.to_string());
                                }
                            }
                            "display" => {
                                if let Some(d) =
                                    param_obj.get("valueString").and_then(|v| v.as_str())
                                {
                                    display = d.to_string();
                                }
                            }
                            "designation" => {
                                if let Some(designation_part) =
                                    param_obj.get("part").and_then(|v| v.as_array())
                                {
                                    if let Ok(des) = self.parse_designation_part(designation_part) {
                                        designation.push(des);
                                    }
                                }
                            }
                            "property" => {
                                // Parse property - implementation would depend on specific property structure
                                // For now, we'll skip detailed property parsing in concept details
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(ConceptDetails {
            name,
            version,
            display,
            designation,
            property,
        })
    }
}

impl Default for DefaultTerminologyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TerminologyProvider for DefaultTerminologyProvider {
    async fn check_valueset_membership(&self, coding: &Coding, valueset_url: &str) -> Result<bool> {
        // Build parameters for $validate-code operation
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "url",
                    "valueUri": valueset_url
                },
                {
                    "name": "system",
                    "valueUri": coding.system
                },
                {
                    "name": "code",
                    "valueCode": coding.code
                }
            ]
        });

        let url = format!("{}/ValueSet/$validate-code", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    // Extract result from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "result" {
                                        if let Some(result_bool) =
                                            param_obj.get("valueBoolean").and_then(|v| v.as_bool())
                                        {
                                            return Ok(result_bool);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Default to false if result not found
                    Ok(false)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn translate_concept(
        &self,
        coding: &Coding,
        conceptmap_url: &str,
        reverse: bool,
    ) -> Result<Vec<FhirPathValue>> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "url",
                    "valueUri": conceptmap_url
                },
                {
                    "name": "system",
                    "valueUri": coding.system
                },
                {
                    "name": "code",
                    "valueCode": coding.code
                },
                {
                    "name": "reverse",
                    "valueBoolean": reverse
                }
            ]
        });

        let url = format!("{}/ConceptMap/$translate", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    let mut translations = Vec::new();

                    // Extract matches from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "match" {
                                        if let Some(match_part) =
                                            param_obj.get("part").and_then(|v| v.as_array())
                                        {
                                            if let Ok(translation) =
                                                self.parse_translation_match(match_part)
                                            {
                                                translations.push(
                                                    TerminologyUtils::translation_to_fhir_value(
                                                        &translation,
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(translations)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn validate_code(&self, system: &str, code: &str) -> Result<bool> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "system",
                    "valueUri": system
                },
                {
                    "name": "code",
                    "valueCode": code
                }
            ]
        });

        let url = format!("{}/CodeSystem/$validate-code", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    // Extract result from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "result" {
                                        if let Some(result_bool) =
                                            param_obj.get("valueBoolean").and_then(|v| v.as_bool())
                                        {
                                            return Ok(result_bool);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(false)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn check_subsumption(&self, coding_a: &Coding, coding_b: &Coding) -> Result<bool> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "system",
                    "valueUri": coding_a.system
                },
                {
                    "name": "codeA",
                    "valueCode": coding_a.code
                },
                {
                    "name": "codeB",
                    "valueCode": coding_b.code
                }
            ]
        });

        let url = format!("{}/CodeSystem/$subsumes", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    // Extract outcome from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "outcome" {
                                        if let Some(outcome) =
                                            param_obj.get("valueCode").and_then(|v| v.as_str())
                                        {
                                            return Ok(outcome == "subsumes");
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(false)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn get_designations(
        &self,
        coding: &Coding,
        language: Option<&str>,
        use_code: Option<&str>,
    ) -> Result<Vec<FhirPathValue>> {
        // Use $lookup operation to get concept details including designations
        let mut params_array = vec![
            json!({
                "name": "system",
                "valueUri": coding.system
            }),
            json!({
                "name": "code",
                "valueCode": coding.code
            }),
        ];

        if let Some(lang) = language {
            params_array.push(json!({
                "name": "language",
                "valueCode": lang
            }));
        }

        let params = json!({
            "resourceType": "Parameters",
            "parameter": params_array
        });

        let url = format!("{}/CodeSystem/$lookup", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    let mut designations = Vec::new();

                    // Extract designations from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "designation" {
                                        if let Some(designation_part) =
                                            param_obj.get("part").and_then(|v| v.as_array())
                                        {
                                            if let Ok(designation) =
                                                self.parse_designation_part(designation_part)
                                            {
                                                // Filter by use code if specified
                                                if let Some(use_filter) = use_code {
                                                    if let Some(ref use_coding) =
                                                        designation.use_coding
                                                    {
                                                        if use_coding.code == use_filter {
                                                            designations.push(TerminologyUtils::designation_to_fhir_value(&designation));
                                                        }
                                                    }
                                                } else {
                                                    designations.push(
                                                        TerminologyUtils::designation_to_fhir_value(
                                                            &designation,
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(designations)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn get_concept_properties(
        &self,
        coding: &Coding,
        property: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "system",
                    "valueUri": coding.system
                },
                {
                    "name": "code",
                    "valueCode": coding.code
                },
                {
                    "name": "property",
                    "valueCode": property
                }
            ]
        });

        let url = format!("{}/CodeSystem/$lookup", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    let mut properties = Vec::new();

                    // Extract properties from Parameters response
                    if let Some(parameter_array) =
                        result.get("parameter").and_then(|v| v.as_array())
                    {
                        for param in parameter_array {
                            if let Some(param_obj) = param.as_object() {
                                if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                    if name == "property" {
                                        if let Some(property_part) =
                                            param_obj.get("part").and_then(|v| v.as_array())
                                        {
                                            if let Ok(property_value) =
                                                self.parse_property_part(property_part, property)
                                            {
                                                properties.push(property_value);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(properties)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn get_terminology_server_url(&self) -> Result<String> {
        Ok(self.server_url.clone())
    }

    async fn expand_valueset(&self, valueset_url: &str) -> Result<Vec<Coding>> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "url",
                    "valueUri": valueset_url
                }
            ]
        });

        let url = format!("{}/ValueSet/$expand", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    let mut codings = Vec::new();

                    // Extract contains from ValueSet expansion
                    if let Some(expansion) = result.get("expansion") {
                        if let Some(contains_array) =
                            expansion.get("contains").and_then(|v| v.as_array())
                        {
                            for contain in contains_array {
                                if let Some(contain_obj) = contain.as_object() {
                                    if let (Some(system), Some(code)) = (
                                        contain_obj.get("system").and_then(|v| v.as_str()),
                                        contain_obj.get("code").and_then(|v| v.as_str()),
                                    ) {
                                        let mut coding = Coding::new(system, code);

                                        if let Some(display) =
                                            contain_obj.get("display").and_then(|v| v.as_str())
                                        {
                                            coding = coding.with_display(display);
                                        }

                                        codings.push(coding);
                                    }
                                }
                            }
                        }
                    }

                    Ok(codings)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0200,
                        format!("Terminology server error: {}", response.status()),
                    ))
                }
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                FP0200,
                format!("Failed to contact terminology server: {}", e),
            )),
        }
    }

    async fn lookup_concept(&self, coding: &Coding) -> Result<Option<ConceptDetails>> {
        let params = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "system",
                    "valueUri": coding.system
                },
                {
                    "name": "code",
                    "valueCode": coding.code
                }
            ]
        });

        let url = format!("{}/CodeSystem/$lookup", self.server_url);

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let result: Value = response.json().await.map_err(|e| {
                        FhirPathError::evaluation_error(
                            FP0200,
                            format!("Failed to parse response: {}", e),
                        )
                    })?;

                    // Parse concept details from lookup result
                    if let Ok(details) = self.parse_concept_details(&result) {
                        Ok(Some(details))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None) // Concept not found
                }
            }
            Err(_) => Ok(None),
        }
    }
}

/// Mock terminology provider for testing purposes
///
/// This provider returns hardcoded results for common test scenarios and
/// doesn't require network connectivity. It's useful for unit testing and
/// development environments.
#[derive(Debug)]
pub struct MockTerminologyProvider;

#[async_trait]
impl TerminologyProvider for MockTerminologyProvider {
    async fn check_valueset_membership(&self, coding: &Coding, valueset_url: &str) -> Result<bool> {
        // Mock implementation - return true for specific test cases
        Ok(coding.system == "http://hl7.org/fhir/administrative-gender"
            && coding.code == "male"
            && valueset_url.contains("administrative-gender"))
    }

    async fn translate_concept(
        &self,
        coding: &Coding,
        _conceptmap_url: &str,
        reverse: bool,
    ) -> Result<Vec<FhirPathValue>> {
        // Mock implementation - return a sample translation
        if coding.code == "M" && !reverse {
            let translation = ConceptTranslation {
                equivalence: "equivalent".to_string(),
                concept: Coding::new("http://hl7.org/fhir/administrative-gender", "male")
                    .with_display("Male"),
                comment: None,
            };
            Ok(vec![TerminologyUtils::translation_to_fhir_value(
                &translation,
            )])
        } else {
            Ok(vec![])
        }
    }

    async fn validate_code(&self, system: &str, code: &str) -> Result<bool> {
        // Mock implementation - validate common codes
        Ok(matches!(
            (system, code),
            (
                "http://hl7.org/fhir/administrative-gender",
                "male" | "female"
            ) | ("http://loinc.org", "789-8")
        ))
    }

    async fn check_subsumption(&self, coding_a: &Coding, coding_b: &Coding) -> Result<bool> {
        // Mock implementation - simple subsumption check
        Ok(coding_a.system == coding_b.system
            && coding_a.code == "parent"
            && coding_b.code == "child")
    }

    async fn get_designations(
        &self,
        coding: &Coding,
        language: Option<&str>,
        _use_code: Option<&str>,
    ) -> Result<Vec<FhirPathValue>> {
        // Mock implementation
        if coding.code == "male" {
            let designation = ConceptDesignation {
                language: language.map(|s| s.to_string()),
                use_coding: None,
                value: match language {
                    Some("es") => "Masculino".to_string(),
                    _ => "Male".to_string(),
                },
            };
            Ok(vec![TerminologyUtils::designation_to_fhir_value(
                &designation,
            )])
        } else {
            Ok(vec![])
        }
    }

    async fn get_concept_properties(
        &self,
        coding: &Coding,
        property: &str,
    ) -> Result<Vec<FhirPathValue>> {
        // Mock implementation
        if coding.code == "male" && property == "definition" {
            Ok(vec![FhirPathValue::String("Male gender".to_string())])
        } else {
            Ok(vec![])
        }
    }

    async fn get_terminology_server_url(&self) -> Result<String> {
        Ok("http://mock-tx.example.com".to_string())
    }

    async fn expand_valueset(&self, _valueset_url: &str) -> Result<Vec<Coding>> {
        Ok(vec![])
    }

    async fn lookup_concept(&self, _coding: &Coding) -> Result<Option<ConceptDetails>> {
        Ok(None)
    }
}
