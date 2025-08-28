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

//! Help system for the REPL

use std::collections::HashMap;
use std::sync::Arc;

use crate::registry::FunctionRegistry;

/// Function help information
pub struct FunctionHelp {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub returns: String,
    pub examples: Vec<String>,
}

/// Help system for REPL functions and commands
pub struct HelpSystem {
    function_help: HashMap<String, FunctionHelp>,
    registry: Option<Arc<FunctionRegistry>>,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    /// Create a new help system with built-in help
    pub fn new() -> Self {
        let mut system = Self {
            function_help: HashMap::new(),
            registry: None,
        };
        
        system.load_builtin_help();
        system
    }

    /// Create a help system with access to function registry
    pub fn with_registry(registry: Arc<FunctionRegistry>) -> Self {
        let mut system = Self {
            function_help: HashMap::new(),
            registry: Some(registry),
        };
        
        system.load_builtin_help();
        system
    }

    /// Get help for a specific function
    pub fn get_function_help(&self, name: &str) -> Option<&FunctionHelp> {
        self.function_help.get(name)
    }

    /// Get function names from registry if available (async version)
    pub async fn get_available_functions(&self) -> Vec<String> {
        if let Some(registry) = &self.registry {
            registry.function_names().await
        } else {
            self.function_help.keys().cloned().collect()
        }
    }

    /// Check if function exists in registry or built-in help (async version)
    pub async fn function_exists(&self, name: &str) -> bool {
        if self.function_help.contains_key(name) {
            return true;
        }
        
        if let Some(registry) = &self.registry {
            registry.has_function(name).await
        } else {
            false
        }
    }

    /// Load built-in function help
    fn load_builtin_help(&mut self) {
        let helps = vec![
            FunctionHelp {
                name: "first".to_string(),
                description: "Returns the first item in a collection".to_string(),
                usage: "collection.first()".to_string(),
                returns: "single item or empty if collection is empty".to_string(),
                examples: vec![
                    "Patient.name.first()".to_string(),
                    "telecom.first().value".to_string(),
                ],
            },
            FunctionHelp {
                name: "last".to_string(),
                description: "Returns the last item in a collection".to_string(),
                usage: "collection.last()".to_string(),
                returns: "single item or empty if collection is empty".to_string(),
                examples: vec![
                    "Patient.name.last()".to_string(),
                    "address.last().city".to_string(),
                ],
            },
            FunctionHelp {
                name: "count".to_string(),
                description: "Returns the number of items in a collection".to_string(),
                usage: "collection.count()".to_string(),
                returns: "Integer".to_string(),
                examples: vec![
                    "Patient.telecom.count()".to_string(),
                    "Bundle.entry.count()".to_string(),
                ],
            },
            FunctionHelp {
                name: "where".to_string(),
                description: "Filters a collection based on a boolean expression".to_string(),
                usage: "collection.where(expression)".to_string(),
                returns: "filtered collection".to_string(),
                examples: vec![
                    "Patient.telecom.where(system = 'email')".to_string(),
                    "Bundle.entry.where(resource.resourceType = 'Patient')".to_string(),
                ],
            },
            FunctionHelp {
                name: "select".to_string(),
                description: "Transforms each item in a collection using an expression".to_string(),
                usage: "collection.select(expression)".to_string(),
                returns: "transformed collection".to_string(),
                examples: vec![
                    "Patient.name.select(given.first())".to_string(),
                    "Bundle.entry.select(resource.id)".to_string(),
                ],
            },
            FunctionHelp {
                name: "exists".to_string(),
                description: "Returns true if the collection is not empty".to_string(),
                usage: "collection.exists() or collection.exists(expression)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.telecom.exists()".to_string(),
                    "Patient.telecom.exists(system = 'email')".to_string(),
                ],
            },
            FunctionHelp {
                name: "empty".to_string(),
                description: "Returns true if the collection is empty".to_string(),
                usage: "collection.empty()".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.telecom.empty()".to_string(),
                    "Patient.photo.empty()".to_string(),
                ],
            },
            FunctionHelp {
                name: "substring".to_string(),
                description: "Extracts a substring from a string".to_string(),
                usage: "string.substring(start) or string.substring(start, length)".to_string(),
                returns: "String".to_string(),
                examples: vec![
                    "Patient.name.first().family.substring(0, 3)".to_string(),
                    "Patient.id.substring(8)".to_string(),
                ],
            },
            FunctionHelp {
                name: "contains".to_string(),
                description: "Returns true if the string contains the given substring".to_string(),
                usage: "string.contains(substring)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.name.first().family.contains('Smith')".to_string(),
                    "Patient.telecom.value.contains('@example.com')".to_string(),
                ],
            },
            FunctionHelp {
                name: "startsWith".to_string(),
                description: "Returns true if the string starts with the given prefix".to_string(),
                usage: "string.startsWith(prefix)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.name.first().family.startsWith('Sm')".to_string(),
                    "Patient.id.startsWith('pat-')".to_string(),
                ],
            },
            FunctionHelp {
                name: "endsWith".to_string(),
                description: "Returns true if the string ends with the given suffix".to_string(),
                usage: "string.endsWith(suffix)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.telecom.value.endsWith('.com')".to_string(),
                    "Patient.name.first().family.endsWith('son')".to_string(),
                ],
            },
            FunctionHelp {
                name: "upper".to_string(),
                description: "Converts a string to uppercase".to_string(),
                usage: "string.upper()".to_string(),
                returns: "String".to_string(),
                examples: vec![
                    "Patient.name.first().family.upper()".to_string(),
                    "Patient.gender.upper()".to_string(),
                ],
            },
            FunctionHelp {
                name: "lower".to_string(),
                description: "Converts a string to lowercase".to_string(),
                usage: "string.lower()".to_string(),
                returns: "String".to_string(),
                examples: vec![
                    "Patient.name.first().family.lower()".to_string(),
                    "Patient.gender.lower()".to_string(),
                ],
            },
            FunctionHelp {
                name: "replace".to_string(),
                description: "Replaces all occurrences of a substring with another string".to_string(),
                usage: "string.replace(old, new)".to_string(),
                returns: "String".to_string(),
                examples: vec![
                    "Patient.name.first().family.replace(' ', '_')".to_string(),
                    "Patient.telecom.value.replace('@old.com', '@new.com')".to_string(),
                ],
            },
            FunctionHelp {
                name: "length".to_string(),
                description: "Returns the length of a string or collection".to_string(),
                usage: "string.length() or collection.length()".to_string(),
                returns: "Integer".to_string(),
                examples: vec![
                    "Patient.name.first().family.length()".to_string(),
                    "Patient.telecom.length()".to_string(),
                ],
            },
            FunctionHelp {
                name: "single".to_string(),
                description: "Returns the single item from a collection, throws error if not exactly one".to_string(),
                usage: "collection.single()".to_string(),
                returns: "single item or error".to_string(),
                examples: vec![
                    "Patient.id.single()".to_string(),
                    "Bundle.entry.where(resource.id = 'patient1').single()".to_string(),
                ],
            },
            FunctionHelp {
                name: "skip".to_string(),
                description: "Skips the first N items in a collection".to_string(),
                usage: "collection.skip(count)".to_string(),
                returns: "collection without first N items".to_string(),
                examples: vec![
                    "Patient.telecom.skip(1)".to_string(),
                    "Bundle.entry.skip(10)".to_string(),
                ],
            },
            FunctionHelp {
                name: "take".to_string(),
                description: "Takes the first N items from a collection".to_string(),
                usage: "collection.take(count)".to_string(),
                returns: "collection with first N items".to_string(),
                examples: vec![
                    "Patient.telecom.take(2)".to_string(),
                    "Bundle.entry.take(10)".to_string(),
                ],
            },
            FunctionHelp {
                name: "distinct".to_string(),
                description: "Returns unique items from a collection".to_string(),
                usage: "collection.distinct()".to_string(),
                returns: "collection with unique items".to_string(),
                examples: vec![
                    "Patient.name.family.distinct()".to_string(),
                    "Bundle.entry.resource.resourceType.distinct()".to_string(),
                ],
            },
            FunctionHelp {
                name: "union".to_string(),
                description: "Combines two collections, removing duplicates".to_string(),
                usage: "collection1.union(collection2)".to_string(),
                returns: "combined collection".to_string(),
                examples: vec![
                    "Patient.name.given.union(Patient.name.family)".to_string(),
                    "Bundle.entry[0].resource.telecom.union(Bundle.entry[1].resource.telecom)".to_string(),
                ],
            },
            FunctionHelp {
                name: "intersect".to_string(),
                description: "Returns items that exist in both collections".to_string(),
                usage: "collection1.intersect(collection2)".to_string(),
                returns: "collection with common items".to_string(),
                examples: vec![
                    "Patient.name[0].given.intersect(Patient.name[1].given)".to_string(),
                ],
            },
            FunctionHelp {
                name: "exclude".to_string(),
                description: "Returns items from the first collection that are not in the second".to_string(),
                usage: "collection1.exclude(collection2)".to_string(),
                returns: "collection with excluded items removed".to_string(),
                examples: vec![
                    "Patient.name.given.exclude('John')".to_string(),
                ],
            },
            FunctionHelp {
                name: "ofType".to_string(),
                description: "Filters collection to items of a specific type".to_string(),
                usage: "collection.ofType(TypeName)".to_string(),
                returns: "filtered collection".to_string(),
                examples: vec![
                    "Bundle.entry.resource.ofType(Patient)".to_string(),
                    "extension.value.ofType(string)".to_string(),
                ],
            },
            FunctionHelp {
                name: "as".to_string(),
                description: "Casts items to a specific type, returns empty if cast fails".to_string(),
                usage: "collection.as(TypeName)".to_string(),
                returns: "cast collection or empty".to_string(),
                examples: vec![
                    "extension.value.as(string)".to_string(),
                    "component.value.as(Quantity)".to_string(),
                ],
            },
            FunctionHelp {
                name: "is".to_string(),
                description: "Checks if items are of a specific type".to_string(),
                usage: "collection.is(TypeName)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "extension.value.is(string)".to_string(),
                    "Bundle.entry.resource.is(Patient)".to_string(),
                ],
            },
            FunctionHelp {
                name: "toInteger".to_string(),
                description: "Converts a value to Integer".to_string(),
                usage: "value.toInteger()".to_string(),
                returns: "Integer or empty if conversion fails".to_string(),
                examples: vec![
                    "'123'.toInteger()".to_string(),
                    "Patient.multipleBirthInteger.toString().toInteger()".to_string(),
                ],
            },
            FunctionHelp {
                name: "toString".to_string(),
                description: "Converts a value to String".to_string(),
                usage: "value.toString()".to_string(),
                returns: "String".to_string(),
                examples: vec![
                    "Patient.multipleBirthInteger.toString()".to_string(),
                    "Patient.active.toString()".to_string(),
                ],
            },
            FunctionHelp {
                name: "iif".to_string(),
                description: "Conditional expression (immediate if)".to_string(),
                usage: "iif(condition, trueValue, falseValue)".to_string(),
                returns: "trueValue if condition is true, falseValue otherwise".to_string(),
                examples: vec![
                    "iif(Patient.active, 'Active', 'Inactive')".to_string(),
                    "iif(Patient.gender = 'male', 'M', 'F')".to_string(),
                ],
            },
            FunctionHelp {
                name: "today".to_string(),
                description: "Returns today's date".to_string(),
                usage: "today()".to_string(),
                returns: "Date".to_string(),
                examples: vec![
                    "today()".to_string(),
                    "Patient.birthDate < today()".to_string(),
                ],
            },
            FunctionHelp {
                name: "now".to_string(),
                description: "Returns current date and time".to_string(),
                usage: "now()".to_string(),
                returns: "DateTime".to_string(),
                examples: vec![
                    "now()".to_string(),
                    "Observation.effectiveDateTime < now()".to_string(),
                ],
            },
            
            // Lambda functions (handled directly by evaluator, not in registry)
            FunctionHelp {
                name: "where".to_string(),
                description: "Filters a collection based on a boolean expression (lambda function)".to_string(),
                usage: "collection.where(condition)".to_string(),
                returns: "filtered collection".to_string(),
                examples: vec![
                    "Patient.telecom.where(system = 'email')".to_string(),
                    "Bundle.entry.where(resource.resourceType = 'Patient')".to_string(),
                    "Patient.name.where(use = 'official')".to_string(),
                ],
            },
            FunctionHelp {
                name: "select".to_string(),
                description: "Transforms each item in a collection using an expression (lambda function)".to_string(),
                usage: "collection.select(expression)".to_string(),
                returns: "transformed collection".to_string(),
                examples: vec![
                    "Patient.name.select(given.first() + ' ' + family)".to_string(),
                    "Bundle.entry.select(resource.id)".to_string(),
                    "Patient.telecom.select(value)".to_string(),
                ],
            },
            FunctionHelp {
                name: "all".to_string(),
                description: "Returns true if all items in the collection satisfy the condition (lambda function)".to_string(),
                usage: "collection.all(condition)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.name.all(family.exists())".to_string(),
                    "Bundle.entry.all(resource.resourceType.exists())".to_string(),
                ],
            },
            FunctionHelp {
                name: "any".to_string(),
                description: "Returns true if any item in the collection satisfies the condition (lambda function)".to_string(),
                usage: "collection.any(condition)".to_string(),
                returns: "Boolean".to_string(),
                examples: vec![
                    "Patient.telecom.any(system = 'email')".to_string(),
                    "Patient.name.any(use = 'official')".to_string(),
                ],
            },
            FunctionHelp {
                name: "repeat".to_string(),
                description: "Repeatedly applies an expression until no new items are found (lambda function)".to_string(),
                usage: "collection.repeat(expression)".to_string(),
                returns: "expanded collection".to_string(),
                examples: vec![
                    "Patient.contact.repeat(name)".to_string(),
                    "Organization.partOf.repeat($this)".to_string(),
                ],
            },
            FunctionHelp {
                name: "aggregate".to_string(),
                description: "Aggregates a collection into a single value using an expression (lambda function)".to_string(),
                usage: "collection.aggregate(aggregator, initial)".to_string(),
                returns: "aggregated result".to_string(),
                examples: vec![
                    "Patient.name.given.aggregate($this + ', ' + $total, '')".to_string(),
                ],
            },
        ];

        for help in helps {
            self.function_help.insert(help.name.clone(), help);
        }
    }
}