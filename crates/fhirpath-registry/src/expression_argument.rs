// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Expression arguments for lambda functions and advanced expression evaluation

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::FhirPathValue;
use std::sync::Arc;
use rustc_hash::FxHashMap;

/// An argument that can be either a pre-evaluated value or an AST expression node
/// This allows functions to receive unevaluated expressions for lambda-style evaluation
#[derive(Debug, Clone)]
pub enum ExpressionArgument {
    /// A pre-evaluated FhirPathValue (traditional function argument)
    Value(FhirPathValue),
    /// An unevaluated expression node for lambda-style functions  
    Expression(Arc<ExpressionNode>),
}

impl ExpressionArgument {
    /// Create from a FhirPathValue
    pub fn value(value: FhirPathValue) -> Self {
        Self::Value(value)
    }
    
    /// Create from an ExpressionNode
    pub fn expression(expr: ExpressionNode) -> Self {
        Self::Expression(Arc::new(expr))
    }
    
    /// Get the value if this is a pre-evaluated argument
    pub fn as_value(&self) -> Option<&FhirPathValue> {
        match self {
            Self::Value(value) => Some(value),
            Self::Expression(_) => None,
        }
    }
    
    /// Get the expression if this is an unevaluated argument
    pub fn as_expression(&self) -> Option<&ExpressionNode> {
        match self {
            Self::Value(_) => None,
            Self::Expression(expr) => Some(expr),
        }
    }
    
    /// Check if this is a value argument
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }
    
    /// Check if this is an expression argument  
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }
}

impl From<FhirPathValue> for ExpressionArgument {
    fn from(value: FhirPathValue) -> Self {
        Self::Value(value)
    }
}

impl From<ExpressionNode> for ExpressionArgument {
    fn from(expr: ExpressionNode) -> Self {
        Self::Expression(Arc::new(expr))
    }
}

/// Variable scope for lambda expression evaluation
/// Manages special variables like $this, $total, $index
#[derive(Debug, Clone)]
pub struct VariableScope {
    /// Standard variables (from context)
    variables: FxHashMap<String, FhirPathValue>,
    /// Lambda-specific variables
    this: Option<FhirPathValue>,
    total: Option<FhirPathValue>, 
    index: Option<i32>,
}

impl VariableScope {
    /// Create a new empty variable scope
    pub fn new() -> Self {
        Self {
            variables: FxHashMap::default(),
            this: None,
            total: None,
            index: None,
        }
    }
    
    /// Create from existing variables map
    pub fn from_variables(variables: FxHashMap<String, FhirPathValue>) -> Self {
        Self {
            variables,
            this: None,
            total: None,
            index: None,
        }
    }
    
    /// Set the $this variable
    pub fn with_this(mut self, this: FhirPathValue) -> Self {
        self.this = Some(this);
        self
    }
    
    /// Set the $total variable
    pub fn with_total(mut self, total: FhirPathValue) -> Self {
        self.total = Some(total);
        self
    }
    
    /// Set the $index variable
    pub fn with_index(mut self, index: i32) -> Self {
        self.index = Some(index);
        self
    }
    
    /// Get a variable value by name
    /// Note: For index, this method returns None since we can't safely return a temporary reference
    /// Use get_owned() for index values
    pub fn get(&self, name: &str) -> Option<&FhirPathValue> {
        match name {
            "this" => self.this.as_ref(),
            "total" => self.total.as_ref(),
            "index" => None, // Cannot return reference to temporary value
            _ => self.variables.get(name),
        }
    }
    
    /// Get a variable value by name, returning owned values
    pub fn get_owned(&self, name: &str) -> Option<FhirPathValue> {
        match name {
            "this" => self.this.clone(),
            "total" => self.total.clone(),
            "index" => self.index.map(|i| FhirPathValue::Integer(i.into())),
            _ => self.variables.get(name).cloned(),
        }
    }
    
    /// Set a regular variable
    pub fn set(&mut self, name: String, value: FhirPathValue) {
        self.variables.insert(name, value);
    }
    
    /// Get all variables as a map for context creation
    pub fn to_variables_map(&self) -> FxHashMap<String, FhirPathValue> {
        let mut map = self.variables.clone();
        
        if let Some(this) = &self.this {
            map.insert("this".to_string(), this.clone());
        }
        if let Some(total) = &self.total {
            map.insert("total".to_string(), total.clone());
        }
        if let Some(index) = &self.index {
            map.insert("index".to_string(), FhirPathValue::Integer((*index).into()));
        }
        
        map
    }
}

impl Default for VariableScope {
    fn default() -> Self {
        Self::new()
    }
}