# UR-LM-001: League Management

**Status:** Draft
**Parent:** Spec Section 4.1 (League CRUD)
**Priority:** Must-have

## Description

A League Operator can create, edit, archive, and un-archive leagues through the web UI. Each league has a name, description, a selected rating algorithm, and a visibility setting (public or private). Public leagues are visible to all authenticated users. Private leagues are visible only to Admins, assigned League Operators, and Player/Viewers whose player record is a member of that league. Visibility is set at creation and can be changed by an authorized operator or Admin. Archiving freezes a league (no new matches or seasons) while preserving all data for read access. Un-archiving resumes write access.

## Rationale

Leagues are the top-level organizational unit for all competitive activity. Operators need full lifecycle control to set up new competitions, modify metadata as needed, and retire leagues without losing historical data. Archived league data must remain visible in aggregate stats and dashboards.

## Acceptance Criteria

- [ ] Operator can create a league by providing a name, description, and selecting one of Elo, Glicko-2, or TrueSkill as the rating algorithm
- [ ] Creating a league automatically starts the first season with the selected algorithm and its default parameters
- [ ] Operator can edit league name and description after creation
- [ ] Operator can archive an active league, which prevents creation of new matches, new seasons, and new players in that league
- [ ] Operator can un-archive an archived league, restoring write access
- [ ] Archived leagues remain visible in league listings and their data is included in aggregate statistics
- [ ] League listing supports filtering by status (active, archived, all)
- [ ] Attempting to record a match, add a player, or start a season in an archived league returns a clear error
- [ ] Editing a league covers name, description, and algorithm parameters within the same algorithm type only (standard edit form)
- [ ] Changing a league's algorithm type is a separate action from editing league metadata, with its own confirmation flow
- [ ] The algorithm type change action triggers a season transition and presents the seeding choice (reset to defaults or seed from prior rankings)
- [ ] Operator can set league visibility to public or private at creation time; omitting the setting defaults to public
- [ ] Operator or Admin can change league visibility from public to private or from private to public after creation
- [ ] Public leagues appear in the league list for all authenticated users
- [ ] Private leagues appear in the league list only for Admins, assigned League Operators, and Player/Viewers whose player record belongs to that league
- [ ] A Player/Viewer with no linked player record sees only public leagues

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: League Management

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And "alice" is assigned as operator of league "Beta League"
    And a user "admin" with role "admin" exists and is authenticated separately

  Scenario: Operator creates a league with Elo algorithm
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues with name "Alpha League", description "A test league", algorithm "elo", and visibility "public"
    Then the response status is 201 Created
    And the response body contains league name "Alpha League"
    And the response body contains algorithm "elo"
    And the response body contains visibility "public"
    And a first season is automatically created for "Alpha League" with algorithm "elo" and default Elo parameters

  Scenario: Operator creates a league with Glicko-2 algorithm and private visibility
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues with name "Glicko League", algorithm "glicko2", and visibility "private"
    Then the response status is 201 Created
    And the response body contains visibility "private"
    And a first season is automatically created for "Glicko League" with algorithm "glicko2"

  Scenario: Operator creates a league with TrueSkill algorithm, omitting visibility
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues with name "TS League" and algorithm "trueskill" and no visibility field
    Then the response status is 201 Created
    And the response body contains visibility "public"

  Scenario: Operator edits league name and description
    Given "alice" is authenticated
    And "alice" has created a league with id 42 named "Beta League"
    When "alice" sends PATCH /api/leagues/42 with name "Beta League v2" and description "Updated description"
    Then the response status is 200 OK
    And the response body contains name "Beta League v2"
    And the response body contains description "Updated description"

  Scenario: Operator archives an active league
    Given "alice" is authenticated
    And league 42 "Beta League" has status "active"
    When "alice" sends POST /api/leagues/42/archive
    Then the response status is 200 OK
    And league 42 status is "archived"

  Scenario: Operator un-archives an archived league
    Given "alice" is authenticated
    And league 42 "Beta League" has status "archived"
    When "alice" sends POST /api/leagues/42/unarchive
    Then the response status is 200 OK
    And league 42 status is "active"

  Scenario: Archived league data appears in league listing
    Given league 42 "Beta League" has status "archived"
    And "alice" is authenticated
    When "alice" sends GET /api/leagues?status=archived
    Then the response status is 200 OK
    And the response body contains league "Beta League" with status "archived"

  Scenario: League listing supports filtering by status
    Given league 1 "Active League" has status "active"
    And league 2 "Dead League" has status "archived"
    And "alice" is authenticated
    When "alice" sends GET /api/leagues?status=active
    Then the response body contains "Active League"
    And the response body does not contain "Dead League"
    When "alice" sends GET /api/leagues?status=archived
    Then the response body contains "Dead League"
    And the response body does not contain "Active League"
    When "alice" sends GET /api/leagues?status=all
    Then the response body contains "Active League"
    And the response body contains "Dead League"

  Scenario: Recording a match in an archived league is rejected
    Given league 42 "Beta League" has status "archived"
    And season 7 belongs to league 42 with end_date set
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with valid participants
    Then the response status is 409 Conflict
    And the response body contains error code "SEASON_CLOSED"

  Scenario: Adding a player to an archived league is rejected
    Given league 42 "Beta League" has status "archived"
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players with name "Bob" and type "human"
    Then the response status is 409 Conflict
    And the response body contains an error indicating the league is archived

  Scenario: Starting a new season in an archived league is rejected
    Given league 42 "Beta League" has status "archived"
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/seasons with algorithm "trueskill"
    Then the response status is 409 Conflict
    And the response body contains an error indicating the league is archived

  Scenario: Operator can change league visibility from public to private
    Given league 42 "Beta League" has visibility "public"
    And "alice" is authenticated and assigned to league 42
    When "alice" sends PATCH /api/leagues/42 with visibility "private"
    Then the response status is 200 OK
    And league 42 visibility is "private"

  Scenario: Admin can change league visibility from private to public
    Given league 42 "Secret League" has visibility "private"
    And "admin" is authenticated
    When "admin" sends PATCH /api/leagues/42 with visibility "public"
    Then the response status is 200 OK
    And league 42 visibility is "public"

  Scenario: Public league appears in listing for all authenticated users
    Given league 10 "Open League" has visibility "public"
    And a user "viewer" with role "viewer" exists and is authenticated
    When "viewer" sends GET /api/leagues
    Then the response body contains "Open League"

  Scenario: Private league appears in listing for Admin but not for unlinked viewer
    Given league 11 "Private League" has visibility "private"
    And a user "viewer" with role "viewer" and no linked player record exists and is authenticated
    When "viewer" sends GET /api/leagues
    Then the response body does not contain "Private League"
    When "admin" sends GET /api/leagues
    Then the response body contains "Private League"

  Scenario: Private league appears for viewer whose player record is a member
    Given league 11 "Private League" has visibility "private"
    And player "Charlie" is a member of league 11
    And user "charlie_user" with role "viewer" is linked to player "Charlie"
    And "charlie_user" is authenticated
    When "charlie_user" sends GET /api/leagues
    Then the response body contains "Private League"

  Scenario: Player/Viewer with no linked player record sees only public leagues
    Given league 11 "Private League" has visibility "private"
    And league 10 "Open League" has visibility "public"
    And a user "viewer" with role "viewer" and no linked player record is authenticated
    When "viewer" sends GET /api/leagues
    Then the response body contains "Open League"
    And the response body does not contain "Private League"

  Scenario: Unauthenticated request to create league is rejected
    When an unauthenticated client sends POST /api/leagues with name "Hack League" and algorithm "elo"
    Then the response status is 401 Unauthorized

  Scenario: Viewer cannot create a league
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/leagues with name "Viewer League" and algorithm "elo"
    Then the response status is 403 Forbidden

  Scenario: Operator cannot edit a league they are not assigned to
    Given "alice" is authenticated and is NOT assigned to league 99
    When "alice" sends PATCH /api/leagues/99 with name "Stolen Name"
    Then the response status is 403 Forbidden

  Scenario: Algorithm type change action triggers season transition with seeding confirmation
    Given "alice" is authenticated and assigned to league 42
    And league 42 current season uses algorithm "elo"
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "trueskill" and seeding_choice "ordinal"
    Then the response status is 200 OK
    And the prior season for league 42 now has a non-null end_date
    And a new season for league 42 is created with algorithm "trueskill"
    And the new season records seeding_choice "ordinal"
```
