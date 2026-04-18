# SR-AUTH-006: League Visibility Enforcement

**Status:** Draft
**Parent:** UR-LM-001, RQ-R3-7
**Priority:** Must-have

## Description

Each league has a visibility setting of either `public` or `private`. Public leagues are visible to all authenticated users. Private leagues are visible only to Admins, League Operators assigned to that league, and Player/Viewers whose linked player record is a member of that league. The visibility setting is configured at league creation and can be changed by an authorized operator or Admin at any time.

## Rationale

Not all leagues should be visible to all users of a deployment. Private leagues allow organizations to run restricted competitions (internal tournaments, invite-only ladders) within the same platform instance without exposing them to unrelated participants. Visibility is enforced server-side to prevent client-side bypasses.

## Acceptance Criteria

- [ ] The `leagues` table includes a `visibility` field with values `public` or `private`; default at creation is `public` unless specified otherwise
- [ ] The league creation endpoint accepts a `visibility` parameter; omitting it defaults to `public`
- [ ] The league update endpoint allows an Admin or assigned League Operator to change `visibility` from `public` to `private` or vice versa
- [ ] The leagues list endpoint filters results server-side based on the requesting user's visibility entitlements:
  - Admin: sees all leagues (public and private)
  - League Operator: sees all public leagues + private leagues they are assigned to
  - Player/Viewer: sees all public leagues + private leagues where their linked player record is a member of that league
  - Player/Viewer with no linked player record: sees all public leagues only
- [ ] A direct GET request to a private league the requesting user is not entitled to see returns a 404 Not Found (not 403, to avoid confirming existence)
- [ ] The visibility entitlement check is enforced on every endpoint that returns league data, not just the league list endpoint
- [ ] Archived private leagues retain their visibility setting; archiving does not make a private league public
- [ ] An Admin changing a league from private to public makes it immediately visible to all authenticated users in subsequent requests
- [ ] An Admin changing a league from public to private immediately removes it from the league list for unauthorized users in subsequent requests

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: League Visibility Enforcement

  Background:
    Given the system is running
    And the following users exist:
      | User    | Role            | Linked Player   |
      | admin1  | Admin           | none            |
      | op1     | League Operator | none            |
      | viewer1 | Player/Viewer   | player-alice    |
      | viewer2 | Player/Viewer   | none            |
    And league "Public League" exists with visibility = "public"
    And league "Private League" exists with visibility = "private"
    And op1 is assigned to "Private League"
    And player-alice is a member of "Private League"

  Scenario: League creation defaults visibility to public when no parameter is supplied
    Given op1 is authenticated and assigned as Admin for this test
    When admin1 creates a league without specifying the visibility parameter
    Then the new league's visibility is "public"

  Scenario: League creation accepts explicit visibility=private
    When admin1 creates a league with visibility = "private"
    Then the new league's visibility is "private"

  Scenario: Admin sees all leagues including private ones in the league list
    Given admin1 is authenticated
    When admin1 requests GET /leagues
    Then "Public League" appears in the response
    And "Private League" appears in the response

  Scenario: League Operator sees public leagues and assigned private leagues
    Given op1 is authenticated
    When op1 requests GET /leagues
    Then "Public League" appears in the response
    And "Private League" appears in the response
    And unassigned private leagues do not appear

  Scenario: League Operator does not see private leagues they are not assigned to
    Given op1 is not assigned to "Unassigned Private League" which exists with visibility = "private"
    And op1 is authenticated
    When op1 requests GET /leagues
    Then "Unassigned Private League" does not appear in the response

  Scenario: Player/Viewer with linked player sees private leagues where their player is a member
    Given viewer1 is authenticated and player-alice is a member of "Private League"
    When viewer1 requests GET /leagues
    Then "Public League" appears in the response
    And "Private League" appears in the response

  Scenario: Player/Viewer direct access to a private league they belong to succeeds
    Given viewer1 is authenticated
    When viewer1 requests GET /leagues/private-league
    Then the HTTP response status is 200

  Scenario: Player/Viewer without linked player sees only public leagues
    Given viewer2 is authenticated and has no linked player record
    When viewer2 requests GET /leagues
    Then "Public League" appears in the response
    And "Private League" does not appear in the response

  Scenario: Direct GET to private league by unauthorized user returns 404, not 403
    Given viewer2 is authenticated and has no linked player record
    When viewer2 requests GET /leagues/private-league
    Then the HTTP response status is 404
    And the HTTP response status is not 403

  Scenario: Direct GET to private league by unauthenticated user returns 404
    Given no session cookie is present
    When a client requests GET /leagues/private-league
    Then the HTTP response status is 404 or 401
    And the HTTP response status is not 403

  Scenario: Admin changes league from private to public — immediately visible to all
    Given admin1 is authenticated
    When admin1 PATCHes /leagues/private-league with visibility = "public"
    Then the HTTP response status is 200
    And viewer2 can subsequently request GET /leagues/private-league and receive HTTP 200

  Scenario: Admin changes league from public to private — immediately hidden from unauthorized users
    Given "Public League" has visibility = "public"
    And admin1 is authenticated
    When admin1 PATCHes /leagues/public-league with visibility = "private"
    Then the HTTP response status is 200
    And viewer2's subsequent request to GET /leagues/public-league returns 404

  Scenario: Archived private league retains its private visibility
    Given "Private League" is archived by admin1
    When viewer2 requests GET /leagues/private-league
    Then the HTTP response status is 404

  Scenario: Visibility check is enforced on match and player endpoints, not just league list
    Given viewer2 is authenticated and has no linked player record
    When viewer2 requests GET /leagues/private-league/leaderboard
    Then the HTTP response status is 404
    When viewer2 requests GET /leagues/private-league/players
    Then the HTTP response status is 404

  Scenario: League Operator can change visibility of their assigned league
    Given op1 is authenticated and assigned to "Public League"
    When op1 PATCHes /leagues/public-league with visibility = "private"
    Then the HTTP response status is 200
```
