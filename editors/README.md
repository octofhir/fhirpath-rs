# FHIRPath Editor Integrations

This directory contains editor plugins and extensions for FHIRPath language support across multiple editors.

## Overview

All editor integrations connect to the `fhirpath-lsp` Language Server Protocol (LSP) server to provide:

- **Syntax Highlighting**: Semantic token-based highlighting
- **IntelliSense**: Context-aware code completion
- **Real-time Diagnostics**: Instant error detection
- **Hover Documentation**: Inline docs with examples
- **Inlay Hints**: Type annotations and parameter names
- **Code Actions**: Quick fixes and refactorings
- **Go to Definition**: Navigate to file references
- **Document Symbols**: Outline view for navigation

## Supported Editors

### 1. Zed Editor

**Directory**: `zed-fhirpath/`  
**Status**: ✅ Complete  
**Files**: 4

Simple, declarative extension using TOML configuration and Tree-sitter highlighting.

**Key Files**:
- `extension.toml` - Extension manifest
- `languages/fhirpath/config.toml` - Language configuration
- `languages/fhirpath/highlights.scm` - Tree-sitter highlighting rules
- `README.md` - User documentation

**Installation**:
```bash
# Package extension
cd zed-fhirpath
tar czf zed-fhirpath-0.1.0.tar.gz .
# Install via Zed extensions UI
```

**Documentation**: [zed-fhirpath/README.md](./zed-fhirpath/README.md)

---

### 2. Visual Studio Code

**Directory**: `vscode-fhirpath/`  
**Status**: ✅ Complete  
**Files**: 8

Full-featured TypeScript extension with LSP client and TextMate grammar.

**Key Files**:
- `package.json` - Extension manifest with dependencies
- `src/extension.ts` - TypeScript LSP client
- `syntaxes/fhirpath.tmLanguage.json` - TextMate grammar
- `language-configuration.json` - Brackets, comments, folding
- `tsconfig.json` - TypeScript configuration
- `.eslintrc.json` - Linting configuration
- `README.md` - User documentation

**Build & Package**:
```bash
cd vscode-fhirpath
npm install
npm run compile
npx vsce package
# Creates fhirpath-0.1.0.vsix
```

**Documentation**: [vscode-fhirpath/README.md](./vscode-fhirpath/README.md)

---

### 3. IntelliJ IDEA

**Directory**: `idea-fhirpath/`  
**Status**: ✅ Complete  
**Files**: 10

Kotlin-based plugin with LSP integration (requires IntelliJ IDEA Ultimate).

**Key Files**:
- `build.gradle.kts` - Gradle build configuration
- `src/main/resources/META-INF/plugin.xml` - Plugin descriptor
- `src/main/kotlin/com/octofhir/fhirpath/` - Kotlin implementation
  - `FhirPathLanguage.kt` - Language definition
  - `FhirPathFileType.kt` - File type
  - `FhirPathParserDefinition.kt` - Parser (minimal, LSP handles parsing)
  - `FhirPathLspServerSupportProvider.kt` - LSP integration
- `README.md` - User documentation

**Build & Package**:
```bash
cd idea-fhirpath
./gradlew buildPlugin
# Creates build/distributions/fhirpath-idea-plugin-0.1.0.zip
```

**Documentation**: [idea-fhirpath/README.md](./idea-fhirpath/README.md)

---

## Prerequisites

All editor integrations require the `fhirpath-lsp` binary to be installed and available in PATH.

### Install LSP Server

**From GitHub Releases**:
```bash
# macOS
curl -L https://github.com/octofhir/fhirpath-rs/releases/latest/download/fhirpath-lsp-macos-x86_64 -o fhirpath-lsp
chmod +x fhirpath-lsp
sudo mv fhirpath-lsp /usr/local/bin/

# Linux
curl -L https://github.com/octofhir/fhirpath-rs/releases/latest/download/fhirpath-lsp-linux-x86_64 -o fhirpath-lsp
chmod +x fhirpath-lsp
sudo mv fhirpath-lsp /usr/local/bin/
```

**Or Build from Source**:
```bash
cd ../crates/fhirpath-lsp
cargo build --release
sudo cp ../../target/release/fhirpath-lsp /usr/local/bin/
```

**Verify Installation**:
```bash
which fhirpath-lsp
fhirpath-lsp --version
```

---

## Configuration

All editors support configuration via `.fhirpath-lsp.toml` in workspace root:

```toml
# FHIR version
fhir_version = "r5"  # Options: "r4", "r4b", "r5"

# Model provider settings
[model_provider]
cache_schemas = true
max_cache_size = 100

# LSP features (toggle on/off)
[features]
diagnostics = true
semantic_tokens = true
completion = true
hover = true
inlay_hints = true
code_actions = true
goto_definition = true

# Performance tuning
[performance]
diagnostic_debounce_ms = 300
max_document_size_kb = 1024
```

---

## File Format

All editors support `.fhirpath` files with the following syntax:

```fhirpath
/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example",
 *   "name": [{"family": "Smith", "given": ["John"]}]
 * }
 */

/**
 * @input-file ./examples/patient.json
 */

// Multiple expressions separated by semicolons
Patient.name.family;
Patient.name.given.first();
Patient.birthDate <= today() - 18 years
```

### Directive Support

- `/** @input {...} */` - Inline FHIR resource as JSON
- `/** @input-file path */` - External resource file reference

---

## Feature Matrix

| Feature | Zed | VS Code | IntelliJ |
|---------|-----|---------|----------|
| Syntax Highlighting | ✅ | ✅ | ✅ |
| Real-time Diagnostics | ✅ | ✅ | ✅ |
| Code Completion | ✅ | ✅ | ✅ |
| Hover Documentation | ✅ | ✅ | ✅ |
| Inlay Hints | ✅ | ✅ | ✅ |
| Code Actions | ✅ | ✅ | ✅ |
| Go to Definition | ✅ | ✅ | ✅ |
| Document Symbols | ✅ | ✅ | ✅ |
| Configuration UI | Basic | Advanced | Advanced |
| File Path Autocomplete | ✅ | ✅ | ✅ |

---

## Development

### Project Structure

```
editors/
├── zed-fhirpath/          # Zed extension
│   ├── extension.toml
│   ├── languages/
│   │   └── fhirpath/
│   │       ├── config.toml
│   │       └── highlights.scm
│   └── README.md
│
├── vscode-fhirpath/       # VS Code extension
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   └── extension.ts
│   ├── syntaxes/
│   │   └── fhirpath.tmLanguage.json
│   ├── language-configuration.json
│   └── README.md
│
├── idea-fhirpath/         # IntelliJ plugin
│   ├── build.gradle.kts
│   ├── src/main/
│   │   ├── kotlin/com/octofhir/fhirpath/
│   │   └── resources/META-INF/plugin.xml
│   └── README.md
│
└── README.md              # This file
```

### Adding New Editor Support

To add support for a new editor:

1. Create directory: `editors/<editor-name>/`
2. Implement LSP client for the editor
3. Configure to spawn `fhirpath-lsp` binary
4. Add syntax highlighting (TextMate/Tree-sitter/etc.)
5. Add configuration support for `.fhirpath-lsp.toml`
6. Document installation and usage in README.md
7. Test all LSP features

---

## Testing

### Manual Testing Checklist

For each editor:

- [ ] Extension/plugin installs successfully
- [ ] LSP server starts when opening `.fhirpath` file
- [ ] Syntax highlighting displays correctly
- [ ] Diagnostics show for syntax errors
- [ ] Code completion triggers on `.`, `(`, `$`
- [ ] Hover shows documentation
- [ ] Inlay hints display (if enabled)
- [ ] Code actions appear for fixable errors
- [ ] Go to definition works for `@input-file` paths
- [ ] Document symbols show in outline/structure panel
- [ ] Configuration changes reload server
- [ ] Extension uninstalls cleanly

### Test Files

Use test files from project:
```bash
# Create test file
cat > test.fhirpath << 'EOF'
/** @input {"resourceType": "Patient", "id": "test"} */

Patient.name.family;
Patient.birthDate
EOF
```

---

## Distribution

### Release Artifacts

Each release should include:

1. **Zed Extension**: `zed-fhirpath-0.1.0.tar.gz`
2. **VS Code Extension**: `fhirpath-0.1.0.vsix`
3. **IntelliJ Plugin**: `fhirpath-idea-plugin-0.1.0.zip`

### Build All Artifacts

```bash
# From editors/ directory

# Zed
cd zed-fhirpath
tar czf ../zed-fhirpath-0.1.0.tar.gz .
cd ..

# VS Code
cd vscode-fhirpath
npm install
npm run compile
npx vsce package
mv fhirpath-0.1.0.vsix ..
cd ..

# IntelliJ
cd idea-fhirpath
./gradlew buildPlugin
cp build/distributions/fhirpath-idea-plugin-0.1.0.zip ..
cd ..
```

---

## Troubleshooting

### Common Issues

**"fhirpath-lsp not found"**
- Ensure binary is in PATH: `which fhirpath-lsp`
- Try absolute path in editor settings

**LSP server won't start**
- Check server version: `fhirpath-lsp --version`
- Test manually: `fhirpath-lsp` (should wait for input)
- Check editor logs for error messages

**No syntax highlighting**
- Verify file extension is `.fhirpath`
- Reload/restart editor
- Check extension is enabled

**Features not working**
- Verify LSP server is running (check editor output panel)
- Check `.fhirpath-lsp.toml` syntax
- Enable verbose logging to diagnose

---

## Contributing

To contribute improvements to editor integrations:

1. Fork repository
2. Make changes in `editors/<editor-name>/`
3. Test thoroughly with checklist above
4. Update task files in `tasks/PHASE_4_TASK_*.md`
5. Submit pull request

---

## License

All editor integrations are licensed under MIT OR Apache-2.0, same as the main project.

---

## Support

For issues or questions:
- **GitHub Issues**: https://github.com/octofhir/fhirpath-rs/issues
- **Documentation**: See LSP_IMPLEMENTATION_PLAN.md
- **Editor-specific docs**: See README.md in each editor directory

---

**Phase 4 Complete** ✅

All three editor integrations implemented and documented. Ready for Phase 5 (Testing & Documentation).
