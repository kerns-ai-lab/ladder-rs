# NFR-SEC-003: Input Sanitization

**Status:** Draft
**Parent:** UR-AUTH-001, UR-LM-001, UR-PM-001, RQ-R3-5
**Priority:** Must-have

## Description

All user-supplied free-text strings are sanitized before storage and before output to prevent cross-site scripting (XSS). Sanitization means: HTML tags are stripped, and characters with special meaning in HTML (`<`, `>`, `&`, `"`, `'`) are escaped or rejected. This applies to all free-text fields across the system (player names, league names, league descriptions, and any other user-controlled string fields).

## Rationale

Unsanitized user input stored in the database and later rendered in the browser is the primary source of stored XSS vulnerabilities. An attacker who can store a script tag in a player name or league description can execute arbitrary JavaScript in any browser that views that name. Because the platform renders operator- and player-supplied content, all free-text inputs must be treated as untrusted.

## Acceptance Criteria

- [ ] All API endpoints that accept free-text string inputs (player name, league name, league description, display name, and any future free-text fields) strip HTML tags from the input before storing it in the database
- [ ] HTML special characters (`<`, `>`, `&`, `"`, `'`) in stored strings are escaped to their HTML entities when the server renders them into HTML responses; they are returned as-is (unescaped) in JSON API responses (the client is responsible for safe rendering)
- [ ] A player or league name submitted as `<script>alert(1)</script>` is stored as either an empty string (tags stripped) or as `&lt;script&gt;alert(1)&lt;/script&gt;` (escaped), never as a raw script tag
- [ ] The sanitization is applied consistently on all write endpoints, not only on specific forms or fields
- [ ] Sanitization does not reject or truncate legitimate Unicode characters, names with apostrophes (e.g., `O'Brien`), or names with dashes and underscores
- [ ] Sanitization does not alter numeric fields, enumerated fields (player_type, algorithm), or binary fields; it applies only to free-text string fields
- [ ] The server never reflects unsanitized user input directly in error messages that are rendered as HTML

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Input Sanitization

  Background:
    Given the system is running
    And a League Operator "op1" is authenticated

  Scenario: Player name containing a script tag is stored without the raw script tag
    When op1 creates a player with name "<script>alert(1)</script>"
    Then the HTTP response status is 201
    And the stored player name does not contain the string "<script>"
    And the stored player name is either empty, the tag-stripped remainder, or the HTML-escaped form

  Scenario: League name containing HTML tags is stored sanitized
    When op1 creates a league with name "<b>Bold League</b>"
    Then the stored league name does not contain "<b>" or "</b>"
    And the stored league name does not render as HTML bold when displayed

  Scenario: League description containing a script tag is sanitized before storage
    When op1 creates a league with description "<script>alert('xss')</script>Some description"
    Then the stored league description does not contain "<script>"
    And the description content "Some description" is preserved after sanitization

  Scenario: XSS payload in player name is returned escaped in JSON API responses
    Given a player exists with a sanitized name (e.g., script tags stripped at write time)
    When a client requests GET /leagues/{id}/players/{player_id}
    Then the JSON response "name" field does not contain a raw unescaped "<script>" tag

  Scenario: HTML special characters in legitimate names are not rejected
    When op1 creates a player with name "O'Brien"
    Then the HTTP response status is 201
    And the stored player name is "O'Brien" (apostrophe preserved)

  Scenario: Player name with a dash and underscore is stored correctly
    When op1 creates a player with name "AI_Agent-v2"
    Then the HTTP response status is 201
    And the stored player name is "AI_Agent-v2" (unchanged)

  Scenario: Unicode characters in player names are not altered by sanitization
    When op1 creates a player with name "Ólafur Björnsson"
    Then the HTTP response status is 201
    And the stored player name is "Ólafur Björnsson" (Unicode preserved)

  Scenario: Numeric fields are not affected by sanitization
    When op1 creates a league configuration with K-factor = 32
    Then the stored K-factor is the integer 32
    And sanitization does not alter the numeric value

  Scenario: Enumerated fields are not affected by sanitization
    When op1 creates a player with player_type = "non_human"
    Then the stored player_type is "non_human" (unchanged by sanitization)

  Scenario: Sanitization is applied on player update as well as creation
    Given player "TestPlayer" exists
    When op1 PATCHes the player name to "<img src=x onerror=alert(1)>"
    Then the stored player name does not contain the raw "<img" tag
    And the HTTP response status is 200

  Scenario: Error messages do not reflect unsanitized user input as HTML
    When op1 submits an invalid request with the field value "<script>alert(1)</script>"
    Then the HTTP response status is 400
    And the response body's "message" field does not contain the raw unescaped "<script>" tag
    And if the value is echoed, it is properly escaped in the JSON response

  Scenario: Sanitization is applied consistently across all write endpoints
    When op1 submits "<script>x</script>" as the name in each of:
      | Endpoint                        |
      | POST /leagues                   |
      | POST /leagues/{id}/players      |
      | PATCH /leagues/{id}             |
      | PATCH /leagues/{id}/players/{player_id} |
    Then in every case the stored value does not contain the raw "<script>" tag
    And no write endpoint skips the sanitization step
```
