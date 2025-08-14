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

//! Unified arithmetic operator implementations

pub mod addition;
pub mod subtraction;
pub mod multiplication;
pub mod division;
pub mod div;
pub mod mod_op;

pub use addition::UnifiedAdditionOperator;
pub use subtraction::UnifiedSubtractionOperator;
pub use multiplication::UnifiedMultiplicationOperator;
pub use division::UnifiedDivisionOperator;
pub use div::UnifiedDivOperator;
pub use mod_op::UnifiedModOperator;