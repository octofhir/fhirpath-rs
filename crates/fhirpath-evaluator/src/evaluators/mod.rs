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

//! Specialized evaluators for different types of operations
//!
//! This module contains specialized evaluators that follow the expression-first
//! evaluation pattern, providing clean separation of concerns for different
//! categories of FHIRPath operations.

pub mod arithmetic;
pub mod collection;
pub mod comparison;
pub mod logical;
pub mod navigation;

pub use arithmetic::ArithmeticEvaluator;
pub use collection::CollectionEvaluator;
pub use comparison::ComparisonEvaluator;
pub use logical::LogicalEvaluator;
pub use navigation::NavigationEvaluator;
