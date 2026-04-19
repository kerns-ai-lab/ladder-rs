# SR-AUTH-008: Invite-Gated Registration

**Status:** Draft
**Parent:** UR-AUTH-001, UR-AUTH-003
**Priority:** Must-have

## Description

In v1, new user account creation via `POST /api/auth/register` requires a valid, unclaimed invite token. The token is provided in the request body. Account creation and token claiming are executed atomically — the `users` row is inserted and the `invite_tokens.claimed_at` and `player_account_links` row are written in a single transaction. There is no separate "register then claim" two-step flow for new users.

The system is designed so that the invite token requirement can be relaxed to allow open self-registration in a future version without any endpoint, schema, or client changes.

## Rationale

The platform is single-tenant and internally operated. Open self-registration without email verification would allow any party with the URL to create an account. Invite tokens ensure every account is associated with a known player record from day one. The single-step register+link UX is simpler than the alternative two-step flow and matches the user journey: the invite URL takes the player directly to a registration page that completes both actions. See ADR-0008.

## Acceptance Criteria

- [ ] `POST /api/auth/register` accepts `{ username, email, password, invite_token }` in the request body; all four fields are required in v1
- [ ] The system validates the invite token before creating the account: token must exist, not be expired (`expires_at > now`), and not already claimed (`claimed_at IS NULL`)
- [ ] If the token is invalid, expired, or already claimed, the registration is rejected with a 400 or 409 response; no user account is created
- [ ] On successful registration, the account creation and token claiming are executed in a single atomic transaction: the `users` row is inserted, `invite_tokens.claimed_at` is set, and a `player_account_links` row is inserted linking the new user to the player the token was issued for
- [ ] If any step of the transaction fails, the entire registration is rolled back; no partial state is committed
- [ ] The response on successful registration is 201 Created; the session cookie is NOT set (the user must then log in separately)
- [ ] Existing authenticated users who receive an invite link use `POST /api/auth/invites/{token}/claim` instead; this endpoint links their existing account to the player record without creating a new account
- [ ] Both the registration path and the claim path share the same token validation logic in the Auth Repository
- [ ] The `invite_token` field in the registration request is architecturally positioned as a validation rule (not a structural requirement), enabling future relaxation to optional without endpoint or schema changes

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Invite-Gated Registration

  Background:
    Given the system is running
    And a player record "Alice" exists with id "player-alice"
    And a League Operator has generated an invite token T for player "player-alice"
    And token T expires in 7 days and is unclaimed

  Scenario: New user registers successfully with a valid invite token
    When a client POSTs to /auth/register with:
      | username     | alice-user         |
      | email        | alice@example.com  |
      | password     | SecurePass1!       |
      | invite_token | T                  |
    Then the HTTP response status is 201
    And a user account "alice@example.com" exists in the database
    And invite token T is marked as claimed
    And a player_account_links record links "alice-user" to "player-alice"
    And no session cookie is set in the response

  Scenario: Registration fails if invite_token is missing
    When a client POSTs to /auth/register without an invite_token field
    Then the HTTP response status is 400
    And no user account is created

  Scenario: Registration fails if invite token is expired
    Given token T expired 1 hour ago
    When a client POSTs to /auth/register with token T
    Then the HTTP response status is 409
    And the response indicates the token is expired
    And no user account is created

  Scenario: Registration fails if invite token is already claimed
    Given token T has already been claimed by another user
    When a client POSTs to /auth/register with token T
    Then the HTTP response status is 409
    And the response indicates the token has already been used
    And no user account is created

  Scenario: Registration fails if invite token does not exist
    When a client POSTs to /auth/register with a random non-existent token value
    Then the HTTP response status is 400
    And no user account is created

  Scenario: Registration and token claim are atomic — failure rolls back entirely
    Given the database is configured to reject the player_account_links insert (simulated failure)
    When a client POSTs to /auth/register with valid token T
    Then the HTTP response status is 500
    And no user account was created
    And token T remains unclaimed

  Scenario: Already-authenticated user claims an invite token without re-registering
    Given user "bob" is authenticated with a valid session
    And a player record "Bob-Player" exists with a valid unclaimed token T2
    When bob POSTs to /auth/invites/T2/claim
    Then the HTTP response status is 200
    And a player_account_links record links "bob" to "Bob-Player"
    And no new user account was created

  Scenario: User logs in after registration (session not auto-issued)
    Given a new user registered successfully with token T
    When the user POSTs to /auth/login with their new credentials
    Then the HTTP response status is 200
    And a session cookie is set
```
