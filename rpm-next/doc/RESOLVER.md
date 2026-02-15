# Dependency Resolution

The `resolver` module is responsible for solving dependency graphs and ensuring that all prerequisites for a package installation are met.

## Algorithm

The current implementation uses a recursive dependency walker with a topological sort to provide a valid installation order.

## Version Constraint Matching

Supports standard version operators:

- `=` (Equal)
- `<` (Less than)
- `<=` (Less than or equal)
- `>` (Greater than)
- `>=` (Greater than or equal)

## Conflict Detection

The resolver checks for conflicting packages and provides errors if a resolution is impossible without removing existing packages.

## Future Plans

- **SAT Solver**: Transition to a proper SAT-based solver (like `libsolv`) for more complex resolution scenarios.
- **Deltas**: Implement support for delta-RPM/DEB to minimize download size.
