# SR-AUTH-003: League-Scoped Roles

**Status:** Draft
**Parent:** UR-AUTH-002
**Priority:** Must-have

## Description

The system supports league-scoped role assignments for League Operators. An operator must be explicitly granted access to each league they manage. The system maintains a mapping of operator-to-league assignments and checks this mapping on every league-specific write operation. Admins bypass league-scoping and have implicit access to all leagues.

## Rationale

League-scoping allows delegation of management responsibilities without granting system-wide access. An organization can have multiple league operators, each responsible for a subset of leagues, without risk of one operator interfering with another's leagues.

## Acceptance Criteria

- [ ] The system stores explicit operator-to-league assignments (a join between user accounts and leagues)
- [ ] An Admin can assign a League Operator to one or more leagues
- [ ] An Admin can revoke a League Operator's access to a specific league
- [ ] A League Operator can only perform write operations in leagues they are explicitly assigned to
- [ ] A League Operator attempting to write to an unassigned league receives a 403 Forbidden response
- [ ] A League Operator can read data from any league they are entitled to see per the league visibility rules (SR-AUTH-006); read access is not further scoped beyond visibility entitlements
- [ ] A Player/Viewer's read access to private leagues is scoped to leagues where their linked player record is a member; they cannot read private leagues they do not belong to, even though they are authenticated
- [ ] An Admin is not required to have explicit league assignments; they have implicit access to all leagues
- [ ] When a new league is created by an Admin, the Admin can immediately assign operators to it
- [ ] The API exposes endpoints for listing a user's league assignments and for managing assignments (Admin only)
- [ ] Deleting or archiving a league does not delete the operator-to-league assignments (they become inert)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: League-Scoped Role Assignments

  Background:
    Given the system is running
    And an Admin "admin1" is authenticated
    And league "League A" and league "League B" exist
    And League Operator "op1" is assigned to "League A" only
    And League Operator "op2" is not assigned to either league
    And Player/Viewer "viewer1" has a linked player record that is a member of "League A"
    And Player/Viewer "viewer2" has no linked player record

  Scenario: Admin assigns a League Operator to a league
    When admin1 POSTs to /admin/leagues/league-b/operators with user_id = op2
    Then the HTTP response status is 200
    And op2 is now assigned to "League B"
    And op2 can perform write operations in "League B"

  Scenario: Admin revokes a League Operator's access to a league
    When admin1 DELETEs /admin/leagues/league-a/operators/op1
    Then the HTTP response status is 200
    And op1 can no longer perform write operations in "League A"

  Scenario: League Operator can perform write operations in their assigned league
    Given op1 is authenticated
    When op1 POSTs to /leagues/league-a/players with valid player data
    Then the HTTP response status is 201

  Scenario: League Operator cannot perform write operations in an unassigned league
    Given op1 is authenticated
    When op1 POSTs to /leagues/league-b/players with valid player data
    Then the HTTP response status is 403
    And the error message indicates op1 is not assigned to "League B"

  Scenario: League Operator cannot record matches in an unassigned league
    Given op1 is authenticated
    When op1 POSTs to /leagues/league-b/matches with valid match data
    Then the HTTP response status is 403

  Scenario: Admin has implicit access to all leagues without explicit assignment
    Given admin1 has no explicit league assignments
    When admin1 POSTs to /leagues/league-b/players with valid player data
    Then the HTTP response status is 201

  Scenario: Admin does not need to be assigned to manage any league
    When admin1 PATCHes /leagues/league-a with updated metadata
    And admin1 PATCHes /leagues/league-b with updated metadata
    Then both requests succeed with HTTP 200

  Scenario: Player/Viewer with linked player in League A can read League A data
    Given viewer1 is authenticated and "League A" is private
    When viewer1 requests GET /leagues/league-a/leaderboard
    Then the HTTP response status is 200

  Scenario: Player/Viewer with linked player in League A cannot read private League B
    Given viewer1 is authenticated and "League B" is private and viewer1's player is not a member
    When viewer1 requests GET /leagues/league-b
    Then the HTTP response status is 404

  Scenario: Player/Viewer with no linked player record sees only public leagues
    Given viewer2 is authenticated and "League A" is private
    When viewer2 requests GET /leagues
    Then "League A" does not appear in the response

  Scenario: League Operator can read public leagues regardless of assignment
    Given op2 is not assigned to "League A" and "League A" is public
    When op2 is authenticated and requests GET /leagues/league-a/leaderboard
    Then the HTTP response status is 200

  Scenario: Deleting a league does not delete operator assignments (they become inert)
    Given op1 is assigned to "League A"
    When admin1 archives "League A"
    Then the op1-to-League-A assignment record still exists in the database
    And op1 receives 404 or an archived-league error when attempting to write to "League A"

  Scenario: API endpoint returns the list of a user's league assignments
    Given op1 is assigned to "League A"
    When admin1 requests GET /admin/users/op1/leagues
    Then the response lists "League A" as op1's assigned league

  Scenario: When a new league is created, Admin can immediately assign an operator
    When admin1 creates a new league "League C"
    And admin1 immediately POSTs to /admin/leagues/league-c/operators with user_id = op2
    Then the HTTP response status is 200
    And op2 is assigned to "League C"
```
