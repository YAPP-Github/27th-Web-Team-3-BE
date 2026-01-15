---
description: Reminder to align with Domain-Driven Architecture
trigger: explicit
---

# Architecture Alignment Hook

## Objective
Ensure all code changes adhere to the established Domain-Driven Design (DDD) modular architecture.

## Checklist
Before creating new files or refactoring:

1.  **Domain Isolation:** Does this logic belong to an existing domain (e.g., `ai`, `health`)?
    - If yes, place it in `src/domain/{domain_name}/`.
    - If no, should a new domain be created?
2.  **Layered Responsibility:**
    - `handler.rs`: strictly for HTTP request/response parsing.
    - `service.rs`: purely business logic.
    - `dto.rs`: data transfer objects only.
3.  **Dependency Rule:** Domains should not tightly couple with each other. Use common `global` modules if shared logic is needed.
4.  **Reference:** Check `docs/project_architecture_and_migration.md` for the latest architectural decisions.
