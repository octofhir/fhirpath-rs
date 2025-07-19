# Changelog Automation

This document describes the automated changelog generation system for OctoFHIR FHIRPath.

## Overview

The project uses [git-cliff](https://git-cliff.org/) to automatically generate changelogs based on [Conventional Commits](https://www.conventionalcommits.org/). The changelog follows the [Keep a Changelog](https://keepachangelog.com/) format.

## Conventional Commits

All commit messages should follow the Conventional Commits specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Supported Types

| Type | Description | Changelog Section |
|------|-------------|-------------------|
| `feat` | New features | Added |
| `add` | New features (alternative) | Added |
| `fix` | Bug fixes | Fixed |
| `bug` | Bug fixes (alternative) | Fixed |
| `doc` | Documentation changes | Documentation |
| `perf` | Performance improvements | Performance |
| `refactor` | Code refactoring | Changed |
| `style` | Code style changes | Changed |
| `test` | Test additions/changes | Testing |
| `build` | Build system changes | Build System |
| `ci` | CI/CD changes | Miscellaneous Tasks |
| `chore` | Maintenance tasks | Miscellaneous Tasks |
| `deps` | Dependency updates | Dependencies |
| `breaking` | Breaking changes | Breaking Changes |
| `revert` | Reverted changes | Reverted |

### Examples

```bash
# Feature addition
feat: add support for FHIRPath functions

# Bug fix
fix: resolve null pointer exception in evaluator

# Breaking change
feat!: change API signature for evaluate_expression

# With scope
feat(cli): add new validation command

# With body and footer
fix: correct parsing of nested expressions

The parser now correctly handles deeply nested parentheses
and function calls within complex FHIRPath expressions.

Fixes #123
```

## Automated Workflows

### 1. Changelog Update Workflow (`.github/workflows/changelog.yml`)

**Triggers:**
- Push to `main` branch
- Manual workflow dispatch

**Actions:**
- Generates changelog entries for unreleased changes
- Updates the `[Unreleased]` section in `CHANGELOG.md`
- Commits changes back to the repository

**Note:** Uses `[skip ci]` to prevent infinite loops.

### 2. Release Changelog Generation (integrated in `.github/workflows/release.yml`)

**Triggers:**
- Tag creation matching `v*` pattern
- Manual release workflow dispatch

**Actions:**
- Generates release notes for the specific version
- Updates the full changelog with the new release
- Creates GitHub release with generated changelog content
- Commits the updated changelog

## Configuration

The changelog generation is configured in `cliff.toml`:

### Key Features

- **Keep a Changelog Format**: Follows the standard format with sections like Added, Changed, Fixed, etc.
- **Conventional Commits Parsing**: Automatically categorizes commits based on type
- **Issue Linking**: Automatically converts issue references to GitHub links
- **Breaking Change Detection**: Highlights breaking changes with **BREAKING** marker
- **Commit Filtering**: Skips certain commit types (like dependency updates)

### Customization

To modify the changelog generation:

1. Edit `cliff.toml` to change commit parsing rules
2. Update the template in the `body` section for different formatting
3. Modify `commit_parsers` to add new commit types or change groupings

## Manual Changelog Generation

You can generate changelogs manually using git-cliff:

```bash
# Install git-cliff
cargo install git-cliff

# Generate unreleased changes
git-cliff --unreleased

# Generate changelog for a specific tag
git-cliff --tag v0.2.0

# Update the changelog file
git-cliff --output CHANGELOG.md

# Generate release notes for the latest tag
git-cliff --latest --strip header --strip footer
```

## Best Practices

### For Contributors

1. **Use Conventional Commits**: Always follow the conventional commit format
2. **Be Descriptive**: Write clear, concise commit messages
3. **Group Related Changes**: Use the same type for related commits
4. **Reference Issues**: Include issue numbers when relevant (e.g., `Fixes #123`)
5. **Mark Breaking Changes**: Use `!` after the type for breaking changes

### For Maintainers

1. **Review Generated Changelogs**: Check that automated entries are accurate
2. **Manual Edits**: Edit `CHANGELOG.md` manually if needed before releases
3. **Version Consistency**: Ensure version numbers match across all packages
4. **Release Notes**: Review generated release notes before publishing

## Troubleshooting

### Common Issues

1. **Missing Changelog Entries**
   - Ensure commits follow conventional commit format
   - Check that commit types are configured in `cliff.toml`
   - Verify the commit isn't being filtered out

2. **Workflow Failures**
   - Check GitHub Actions logs for detailed error messages
   - Ensure repository has write permissions for the workflow
   - Verify git-cliff configuration is valid

3. **Duplicate Entries**
   - May occur if commits are manually added to changelog
   - Use `git-cliff --output CHANGELOG.md` to regenerate

### Manual Fixes

If the automated changelog needs correction:

1. Edit `CHANGELOG.md` manually
2. Commit the changes with `docs: fix changelog [skip ci]`
3. The next automated run will preserve manual edits

## Integration with Release Process

The changelog automation is fully integrated with the release process:

1. **Development**: Commits automatically update the `[Unreleased]` section
2. **Release**: Creating a tag triggers changelog generation for that version
3. **GitHub Release**: Release notes are automatically generated from changelog
4. **Documentation**: Updated changelog is committed back to the repository

This ensures that changelogs are always up-to-date and releases have proper documentation of changes.

## Migration from Manual Changelog

If migrating from a manual changelog:

1. Ensure existing entries follow Keep a Changelog format
2. Add the git-cliff footer comment: `<!-- generated by git-cliff -->`
3. Future entries will be automatically generated
4. Manual entries can still be added if needed

## Support

For issues with changelog automation:

1. Check the [git-cliff documentation](https://git-cliff.org/)
2. Review GitHub Actions workflow logs
3. Open an issue in the repository for project-specific problems
