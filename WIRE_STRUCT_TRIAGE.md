# Wire Struct Generation Triage

## Overview

This document provides a comprehensive triage of the `*Wire` struct generation feature in the `simple_endian` crate, implemented via the `#[derive(Endianize)]` macro.

## Current Behavior

### Naming Convention

When `#[derive(Endianize)]` is applied to a struct/enum/union, the macro generates a companion `*Wire` type by appending "Wire" to the original type name:

```rust
#[derive(Endianize)]
#[endian(be)]
struct Header {
    a: u32,
}
// Generates: HeaderWire
```

**Implementation**: `simple_endian_derive/src/endianize.rs`, line 314:
```rust
let wire_name = format_ident!("{}Wire", name);
```

### Generated Wire Type Characteristics

1. **Visibility**: Inherits from the original struct (`#vis`)
2. **Fields**: Wrapped in endian types (`BigEndian<T>` or `LittleEndian<T>`)
3. **Representation**: Defaults to `#[repr(C)]`, customizable via `#[wire_repr(...)]`
4. **Derives**: Optional via `#[wire_derive(...)]`

## Test Coverage

All tests pass (as of 2026-01-03):
- 57 unit tests in `src/lib.rs`
- 52 integration tests across multiple test files
- Examples compile and run successfully

Specific Wire-related tests:
- `tests/derive_endianize.rs`: Basic Wire generation
- `tests/derive_io_wire_auto_impl.rs`: IO trait implementation  
- `tests/derive_wire_derive_passthrough.rs`: Trait passthrough
- Various repr and layout tests

## Potential Issues and Edge Cases

### 1. Name Collision: Structs Already Ending in "Wire"

**Issue**: If a user names their struct `DataWire`, the generated type becomes `DataWireWire`, which is awkward and potentially confusing.

**Example**:
```rust
#[derive(Endianize)]
#[endian(be)]
struct ConfigWire {  // Original name ends in "Wire"
    value: u32,
}
// Generates: ConfigWireWire (awkward)
```

**Status**: Currently compiles without error or warning.

**Recommendation**: 
- Option A: Add a lint warning when struct name ends in "Wire"
- Option B: Smart suffix handling (detect and avoid duplication)
- Option C: Document as a naming convention to avoid

### 2. Consistency with Documentation

**Finding**: The term "*Wire" is used consistently throughout:
- README.md
- LLMs.txt  
- Examples
- Tests

**Note**: The problem statement mentioned "*Wired" but no such naming exists in the codebase. This may have been a typo.

### 3. Visibility and Accessibility

**Current Behavior**: Generated Wire types inherit visibility from the original type.

```rust
pub struct Header { ... }
// Generates: pub struct HeaderWire { ... }

struct Private { ... }
// Generates: struct PrivateWire { ... }
```

**Status**: Working as designed, appropriate for most use cases.

### 4. Generic Support

**Current Status**: The macro supports generics via `&input.generics` and `split_for_impl()`.

**Verification Needed**: Need to confirm complex generic scenarios work correctly.

### 5. Documentation Generation

**Potential Issue**: Generated Wire types don't automatically inherit doc comments from the original type.

**Impact**: Users might need to manually document Wire types for better IDE support.

## Recommendations

### High Priority
1. **Clarify Naming Convention**: Update documentation to explicitly state the "append Wire" behavior
2. **Add Naming Best Practices**: Document recommended naming patterns (avoid ending user structs with "Wire")

### Medium Priority
3. **Enhanced Documentation**: Consider automatic doc comment generation for Wire types
4. **Improved Error Messages**: Add helpful compile errors for unsupported patterns

### Low Priority
5. **Smart Suffix Handling**: Consider detecting and avoiding "WireWire" patterns
6. **Customizable Suffix**: Allow users to override the "Wire" suffix (e.g., `#[wire_suffix = "Format"]`)

## Conclusion

The `*Wire` struct generation feature is working correctly with good test coverage. The primary issue identified is the potential for awkward naming when structs already end in "Wire", which is a minor ergonomic concern rather than a bug.

**Status**: ✅ Feature is functional and well-tested
**Action Items**: Documentation improvements and optional naming enhancements
