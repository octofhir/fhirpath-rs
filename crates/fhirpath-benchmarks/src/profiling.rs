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

//\! Performance profiling utilities

/// Profiler context (placeholder)
pub struct ProfilerContext;

impl ProfilerContext {
    /// Start profiling
    pub fn start() -> Self {
        Self
    }

    /// Generate flamegraph
    pub fn flamegraph(&self) -> Option<String> {
        None
    }
}
