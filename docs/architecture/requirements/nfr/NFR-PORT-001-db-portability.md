# NFR-PORT-001: Database Portability

**Status:** Draft
**Parent:** Spec Section 6 (Database Portability)
**Priority:** Should-have

## Description

The persistence layer uses sqlx and all queries are written to be portable between SQLite and PostgreSQL. Only SQLite is supported for v1 deployment, but the query layer must not use SQLite-specific syntax that would prevent a future PostgreSQL migration. No full repository pattern is required; a thin abstraction layer is sufficient.

## Rationale

While SQLite is adequate for v1 scale targets, future growth or multi-tenant deployment may require PostgreSQL. Writing portable queries from the start avoids a costly migration of the persistence layer later. The thin abstraction approach avoids over-engineering while maintaining portability.

## Acceptance Criteria

- [ ] All database queries use sqlx and compile against both SQLite and PostgreSQL sqlx feature flags
- [ ] No SQLite-specific SQL syntax is used in queries (e.g., no `AUTOINCREMENT`, no SQLite-specific type affinity tricks)
- [ ] No PostgreSQL-specific SQL syntax is used in queries (e.g., no `RETURNING *` if not supported by SQLite, no PostgreSQL-specific functions)
- [ ] Schema migrations are written in portable SQL or use conditional syntax where dialect differences are unavoidable
- [ ] The persistence layer compiles and passes tests with the SQLite sqlx feature enabled
- [ ] A CI check or compile-time verification confirms query portability (sqlx compile-time checked queries against both dialects)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Database Portability

  Background:
    Given the persistence layer uses sqlx for all database access
    And the current deployment target is SQLite

  Scenario: All queries compile successfully with the SQLite sqlx feature flag
    Given the Rust project is compiled with the SQLite sqlx feature enabled
    When `cargo build` is executed
    Then the build succeeds with no compile-time query errors
    And all sqlx queries pass compile-time verification against the SQLite schema

  Scenario: No SQLite-specific AUTOINCREMENT keyword appears in schema migrations
    Given the schema migration files are inspected
    When each migration SQL statement is scanned for SQLite-specific syntax
    Then no statement contains the AUTOINCREMENT keyword (SQLite-only, not valid in PostgreSQL)
    And primary key columns use INTEGER PRIMARY KEY or equivalent portable syntax

  Scenario: No PostgreSQL-specific syntax appears in query files
    Given the Rust source files containing sqlx queries are inspected
    When each query is scanned for PostgreSQL-specific functions or syntax
    Then no query uses PostgreSQL-specific functions (e.g., gen_random_uuid(), ILIKE as dialect-exclusive)
    And no query uses PostgreSQL array syntax (e.g., ARRAY[]) that is unsupported by SQLite

  Scenario: RETURNING clause usage is compliant with both dialects
    Given sqlx queries that use RETURNING clauses are inspected
    When each such query is evaluated for cross-dialect compatibility
    Then any use of RETURNING is limited to forms supported by both SQLite (3.35+) and PostgreSQL
    Or alternatively, no RETURNING clauses are used and separate SELECT queries retrieve inserted data

  Scenario: Schema migrations are written in portable SQL
    Given the migration files are reviewed
    When each migration is inspected for dialect-specific SQL
    Then column type definitions use portable types (INTEGER, TEXT, REAL, BLOB or equivalent)
    And no SQLite-specific type affinity tricks (e.g., bare column definitions relying on type affinity) are used

  Scenario: Queries use parameterized statements, not string interpolation
    Given the Rust source files containing sqlx queries are inspected
    When each query is reviewed
    Then all user-supplied values are passed as sqlx parameters (using ? or $1 placeholders)
    And no query constructs SQL strings through string concatenation or format!() with user data

  Scenario: Database operations succeed at runtime against the SQLite backend
    Given a fresh SQLite database initialized by the schema migrations
    When the full integration test suite is executed against the SQLite backend
    Then all persistence-layer tests pass
    And no SQLite-specific or PostgreSQL-specific query errors are produced
```
