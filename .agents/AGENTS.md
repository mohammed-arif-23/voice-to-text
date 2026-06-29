# Workspace Agent Rules for ScribeRx

## Guidelines & Token Optimization

1. **Contracts over code-reading**: All cross-crate implementations must reference trait contracts in `docs/ARCHITECTURE.md`.
2. **Terse Summaries**: Subagents must return short summaries of changes made rather than full file dumps.
3. **Design Token Compliance**: UI development in `ui/` must strictly follow tokens defined in `docs/DESIGN_SYSTEM.md`.
4. **Safety-Critical Drug Logic**: Numeric dosages must never be silently auto-corrected by matching algorithms.
