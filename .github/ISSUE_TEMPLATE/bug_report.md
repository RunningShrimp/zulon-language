---
name: Bug report
about: Create a report to help us improve
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

**Clear and concise description of the bug**:

<!-- What happened? What did you expect to happen? -->

## Reproduction Steps

**Steps to reproduce the behavior**:

1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Example Code**:
```zulon
// Provide a minimal example that reproduces the bug
extern fn printf(s: &u8, ...) -> i32;

fn main() -> i32 {
    printf("Hello\n");
    0
}
```

**Command Used**:
```bash
# Command you ran
./target/release/zulon-compiler example.zl
```

## Expected Behavior

**What you expected to happen**:
<!-- Describe what should have happened -->

## Actual Behavior

**What actually happened**:
<!-- Describe what actually happened -->
Include error messages, stack traces, etc.

## Environment

**ZULON Version**:
```bash
zulon-compiler --version
# Output: ...
```

**Rust Version**:
```bash
rustc --version
# Output: ...
```

**LLVM Version**:
```bash
llc --version
# Output: ...
```

**Operating System**:
- [ ] macOS
- [ ] Linux
- [ ] Windows
- [ ] Other (specify):

## Additional Context

**Screenshots**:
<!-- If applicable, add screenshots to help explain your problem -->

**Additional context**:
<!-- Any other relevant information about the problem -->

**Related Issues**:
<!-- Link to related issues here -->

## Possible Solution

**Do you have a fix in mind?**:
<!-- If you have an idea for how to fix this, please describe it here -->

**Would you like to submit a PR?**:
- [ ] Yes, I'd like to submit a PR
- [ ] No, I don't have time

---

**Thank you for reporting this bug!** 🐛
