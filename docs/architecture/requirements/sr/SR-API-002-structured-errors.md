# SR-API-002: Structured Error Responses

**Status:** Draft
**Parent:** UR-ME-001, UR-LM-001, UR-PM-001
**Priority:** Must-have

## Description

The REST API returns structured, machine-readable error responses. Each error response includes an error code, a human-readable message, and field-level validation details where applicable. Error responses use a consistent JSON schema across all endpoints.

## Rationale

Structured errors enable the frontend to display specific, actionable feedback to operators (e.g., "K-factor must be between 1 and 100" rather than "Bad request"). Machine-readable error codes allow programmatic error handling by swarm operators. Consistency across endpoints reduces frontend complexity.

## Acceptance Criteria

- [ ] All error responses use a consistent JSON structure containing at minimum: error_code (string), message (string), and an optional details array
- [ ] Validation errors include field-level detail: field name, rejected value, and constraint that was violated
- [ ] Error codes are documented and stable (not random strings)
- [ ] HTTP status codes are used correctly (400 for validation, 404 for not found, 409 for conflicts like duplicates, 422 for semantic errors)
- [ ] The error response structure is the same whether the error originates in the API layer or the library persistence layer
- [ ] Error responses never expose internal implementation details (stack traces, SQL queries, file paths)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Structured Error Responses

  Background:
    Given the system is running
    And a League Operator is authenticated

  Scenario: Validation error response includes error_code, message, and details
    When the League Operator submits a league creation request with an invalid K-factor = -5
    Then the HTTP response status is 400
    And the response Content-Type is "application/json"
    And the response body contains a field "error_code" of type string
    And the response body contains a field "message" of type string
    And the response body contains a field "details" of type array

  Scenario: Field-level validation error includes field name, rejected value, and violated constraint
    When the League Operator submits a configuration update with K-factor = -5
    Then the HTTP response status is 400
    And the "details" array contains at least one entry with:
      | sub-field        | value        |
      | field            | "k_factor"   |
      | rejected_value   | "-5"         |
      | constraint       | a description of the valid range |

  Scenario: HTTP 404 is returned for a resource that does not exist
    When the League Operator requests GET /leagues/nonexistent-id
    Then the HTTP response status is 404
    And the response body contains "error_code" with value "NOT_FOUND" or equivalent

  Scenario: HTTP 409 is returned for a duplicate match submission
    Given a match was already recorded for the same two players, same outcome, and same timestamp
    When the League Operator submits the same match again
    Then the HTTP response status is 409
    And the response body contains "error_code" with a stable conflict code such as "DUPLICATE_MATCH"

  Scenario: HTTP 422 is returned for a semantically invalid request
    When the League Operator submits a match recording request referencing a player who is soft-deleted
    Then the HTTP response status is 422
    And the response body contains "error_code" and "message" fields

  Scenario: Error codes are stable documented strings, not random values
    When the League Operator submits invalid data twice with the same type of validation error
    Then both responses contain the same "error_code" value

  Scenario: Errors from the persistence layer use the same JSON schema as API-layer errors
    Given a unique constraint violation occurs at the database level during match recording
    When the server processes the request
    Then the HTTP response to the client uses the standard structured error JSON schema
    And the response body does not contain raw database error text

  Scenario: Error responses never expose stack traces
    When any API request causes an internal server error
    Then the HTTP response status is 500
    And the response body contains "error_code" and "message"
    And the response body does not contain a stack trace

  Scenario: Error responses never expose SQL query text
    When any API request causes a database error
    Then the response body does not contain SQL statement text

  Scenario: Error responses never expose internal file paths
    When any API request causes a server error
    Then the response body does not contain file system path strings

  Scenario: 403 Forbidden includes structured error with required role indication
    Given a Player/Viewer attempts to record a match
    When the request reaches the authorization layer
    Then the HTTP response status is 403
    And the response body contains "error_code" and "message" indicating insufficient permissions

  Scenario: Error response structure is consistent across all endpoints
    When a validation error occurs on POST /leagues
    And a validation error occurs on POST /leagues/{id}/matches
    And a validation error occurs on PATCH /leagues/{id}
    Then all three responses conform to the same JSON schema with "error_code", "message", and "details"
```
