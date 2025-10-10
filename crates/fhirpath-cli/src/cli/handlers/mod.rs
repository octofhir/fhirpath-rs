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

//! Command handlers for CLI operations

pub mod analyze;
pub mod completions;
pub mod config;
pub mod docs;
pub mod evaluate;
pub mod pipe;
pub mod registry;
pub mod validate;

pub use analyze::handle_analyze;
pub use completions::handle_completions;
pub use config::handle_config;
pub use docs::handle_docs;
pub use evaluate::handle_evaluate;
pub use pipe::{handle_pipe_mode, is_stdin_pipe, is_stdout_pipe};
pub use registry::{
    handle_registry, handle_registry_list_functions, handle_registry_list_operators,
    handle_registry_show,
};
pub use validate::handle_validate;
