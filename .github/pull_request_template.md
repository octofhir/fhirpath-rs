# Pull Request

## Description
Brief description of the changes in this PR.

## Type of Change
Please check the type of change your PR introduces:
- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)  
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation (changes to documentation only)
- [ ] 🔧 Refactor (code change that neither fixes a bug nor adds a feature)
- [ ] ⚡ Performance (code change that improves performance)
- [ ] 🧪 Tests (adding missing tests or correcting existing tests)
- [ ] 🔨 Chore (changes to build process, CI, dependencies, etc.)

## Related Issues
Closes #(issue number)

## Changes Made
- [ ] Change 1
- [ ] Change 2
- [ ] Change 3

## Testing
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] All new and existing tests pass locally with my changes
- [ ] I have run `just qa` and all checks pass

## FHIRPath Compliance
- [ ] Changes maintain compliance with FHIRPath specification
- [ ] Official test suite still passes (if applicable)
- [ ] New functionality includes appropriate test coverage

## Performance Impact
- [ ] No performance impact
- [ ] Performance improvement (include benchmark results)
- [ ] Performance regression (justified and documented)

## Documentation
- [ ] I have updated documentation as needed
- [ ] Code is self-documenting with appropriate comments
- [ ] Public API changes are documented

## Breaking Changes
If this PR introduces breaking changes, please describe them here:
- Breaking change 1
- Breaking change 2

## Migration Guide
If breaking changes exist, provide migration instructions:
```rust
// Before
let old_code = example();

// After  
let new_code = updated_example();
```

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings
- [ ] I have added appropriate error handling
- [ ] Any dependent changes have been merged and published in downstream modules

## Additional Notes
Any additional information that reviewers should know about this PR.