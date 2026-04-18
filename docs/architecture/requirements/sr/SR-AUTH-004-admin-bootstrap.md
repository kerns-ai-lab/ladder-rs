# SR-AUTH-004: Admin Bootstrap

**Status:** Draft
**Parent:** UR-AUTH-001, RQ-R3-1
**Priority:** Must-have

## Description

On the first startup of a new deployment where the user table is empty, the system automatically creates a default admin account with a system-generated username and password. The generated credentials are printed to stdout so the operator can retrieve them from the deployment logs. The default admin is required to change their password immediately on first login before any other action is permitted.

## Rationale

A fresh deployment has no existing admin to create the first account. Without automatic bootstrapping, there is no path to initial access. Printing credentials to stdout follows the convention of secrets-at-rest-in-logs (acceptable for bootstrap-only use) while ensuring the operator can retrieve them. Forcing an immediate password change limits the window of exposure for the generated credential.

## Acceptance Criteria

- [ ] On startup, the system checks whether the user table contains any accounts
- [ ] If the user table is empty, the system generates a default admin account with a random, cryptographically generated password of at least 16 characters
- [ ] The generated username (e.g., `admin`) and password are printed to stdout in plaintext exactly once during the bootstrap startup
- [ ] The bootstrap admin account is assigned the Admin role
- [ ] The bootstrap admin account is marked as requiring a password change on next login
- [ ] When the bootstrap admin logs in for the first time, they are presented with a mandatory password change form before they can access any other part of the application
- [ ] Any API request made by the bootstrap admin (other than the password change endpoint) before the password has been changed is rejected with a 403 response and a message indicating a password change is required
- [ ] After the password is changed, the forced-change flag is cleared and normal access is restored
- [ ] If the user table already contains at least one account on startup, no bootstrap credentials are generated or printed

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Admin Bootstrap on Empty Deployment

  Scenario: Bootstrap creates default admin account on first startup with empty user table
    Given the database user table is empty
    When the application starts for the first time
    Then a default admin account is created automatically
    And the generated username (e.g., "admin") and password are printed to stdout exactly once
    And the generated password is at least 16 characters long
    And the generated password is cryptographically random

  Scenario: Bootstrap admin is assigned the Admin role
    Given the database user table is empty
    When the application starts
    Then the bootstrap admin account has role "Admin"
    And the Admin role grants access to all admin endpoints

  Scenario: Bootstrap admin account is marked for forced password change
    Given the database user table is empty
    When the application starts
    Then the bootstrap admin account has the "force_password_change" flag set to true

  Scenario: Bootstrap admin must change password before accessing any other endpoint
    Given the application has bootstrapped and the admin has not yet changed the password
    When the bootstrap admin logs in with the generated credentials
    And the bootstrap admin requests GET /leagues
    Then the HTTP response status is 403
    And the response message indicates a password change is required before proceeding

  Scenario: Bootstrap admin can access only the password change endpoint before changing password
    Given the application has bootstrapped and the admin has not yet changed the password
    When the bootstrap admin logs in and POSTs to /auth/change-password with a new password
    Then the HTTP response status is 200
    And the forced-change flag is cleared
    And the bootstrap admin can now access GET /leagues with HTTP 200

  Scenario: Bootstrap does not generate credentials when user table already has accounts
    Given the database user table contains at least one account
    When the application starts
    Then no bootstrap credentials are printed to stdout
    And no new admin account is created automatically

  Scenario: Bootstrap credentials are printed exactly once, not on subsequent startups
    Given the application has already bootstrapped on a previous startup
    And the user table contains the bootstrap admin account
    When the application is restarted
    Then no credentials are printed to stdout on the second startup

  Scenario: Bootstrap admin password hash stored securely — not in plaintext
    Given the application bootstrapped and created an admin account
    When the admin table row for the bootstrap account is inspected
    Then the password field contains a hash (bcrypt, argon2, or scrypt), not the plaintext password

  Scenario: After forced password change, normal Admin access is fully restored
    Given the bootstrap admin has changed their password
    When the admin creates a new league via POST /leagues
    Then the HTTP response status is 201

  Scenario: Bootstrap admin cannot bypass the forced-change restriction via any endpoint
    Given the bootstrap admin has not changed their password
    And the bootstrap admin has a valid session
    When the bootstrap admin attempts to access any endpoint except /auth/change-password
    Then every such request returns HTTP 403 with a password-change-required message
```
