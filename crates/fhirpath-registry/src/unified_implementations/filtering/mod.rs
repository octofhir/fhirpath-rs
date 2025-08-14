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

//! Filtering and projection functions

pub mod where_fn;
pub mod where_enhanced;
pub mod select;
pub mod of_type;

// Re-export functions
pub use where_fn::UnifiedWhereFunction;
pub use where_enhanced::EnhancedWhereFunction;
pub use select::UnifiedSelectFunction;
pub use of_type::UnifiedOfTypeFunction;