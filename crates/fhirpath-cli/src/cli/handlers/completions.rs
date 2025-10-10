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

//! Shell completion generation handler

use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

use crate::cli::Cli;

/// Generate shell completions and print to stdout
pub fn handle_completions(shell: Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate(shell, &mut cmd, name, &mut io::stdout());

    Ok(())
}

/// Print installation instructions for shell completions
pub fn print_installation_instructions(shell: Shell) {
    eprintln!("\n# Installation Instructions\n");

    match shell {
        Shell::Bash => {
            eprintln!("Add this to your ~/.bashrc:");
            eprintln!("  eval \"$(octofhir-fhirpath completions bash)\"");
            eprintln!("\nOr save to a file:");
            eprintln!(
                "  octofhir-fhirpath completions bash > ~/.local/share/bash-completion/completions/octofhir-fhirpath"
            );
        }
        Shell::Zsh => {
            eprintln!("Add this to your ~/.zshrc:");
            eprintln!("  eval \"$(octofhir-fhirpath completions zsh)\"");
            eprintln!("\nOr save to a file in your $fpath:");
            eprintln!(
                "  octofhir-fhirpath completions zsh > ~/.zsh/completions/_octofhir-fhirpath"
            );
            eprintln!("  # Then add to ~/.zshrc: fpath=(~/.zsh/completions $fpath)");
        }
        Shell::Fish => {
            eprintln!("Save completions to fish config:");
            eprintln!(
                "  octofhir-fhirpath completions fish > ~/.config/fish/completions/octofhir-fhirpath.fish"
            );
        }
        Shell::PowerShell => {
            eprintln!("Add this to your PowerShell profile:");
            eprintln!(
                "  octofhir-fhirpath completions powershell | Out-String | Invoke-Expression"
            );
            eprintln!("\nTo find your profile location, run:");
            eprintln!("  echo $PROFILE");
        }
        Shell::Elvish => {
            eprintln!("Add this to your ~/.elvish/rc.elv:");
            eprintln!("  eval (octofhir-fhirpath completions elvish)");
        }
        _ => {
            eprintln!("Please refer to your shell's documentation for completion installation.");
        }
    }

    eprintln!();
}
