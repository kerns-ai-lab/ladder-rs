# NFR-SEC-002: Session Security

**Status:** Draft
**Parent:** UR-AUTH-001, RQ-R3-5
**Priority:** Must-have

## Description

Session tokens are issued exclusively as HTTP cookies with the `HttpOnly`, `Secure`, and `SameSite=Strict` attributes set. Session tokens are never placed in response bodies, URL query parameters, or any JavaScript-accessible storage. Session expiry is enforced server-side.

## Rationale

Storing session tokens in `localStorage` or returning them in response bodies exposes them to XSS attacks. `HttpOnly` prevents JavaScript from reading the cookie. `Secure` ensures the cookie is only transmitted over HTTPS, preventing interception on unencrypted connections. `SameSite=Strict` mitigates cross-site request forgery (CSRF) by preventing the cookie from being sent on cross-origin requests.

## Acceptance Criteria

- [ ] The login endpoint sets the session token as a cookie with all three attributes: `HttpOnly`, `Secure`, and `SameSite=Strict`
- [ ] The login response body does not contain the session token value
- [ ] The session token is not included in any URL (neither as a path segment nor as a query parameter) in any response
- [ ] The session token is not stored in `localStorage`, `sessionStorage`, or any other JavaScript-accessible browser storage mechanism
- [ ] The cookie has an explicit `Max-Age` or `Expires` attribute matching the server-side session expiry configuration
- [ ] The server validates session expiry on every authenticated request; an expired session token results in a 401 response even if the cookie is structurally valid
- [ ] The logout endpoint clears the session cookie by setting it with `Max-Age=0` (or equivalent) in addition to invalidating the token server-side
- [ ] Session tokens are not logged in application logs, access logs, or error reports
- [ ] The `Secure` cookie attribute is controlled by the `HTTPS_ENABLED` environment variable; when `HTTPS_ENABLED=false` (development mode), the `Secure` attribute is omitted so that session cookies are transmitted over HTTP localhost; `HttpOnly` and `SameSite=Strict` remain set regardless of `HTTPS_ENABLED`

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Session Security

  Background:
    Given the system is running over HTTPS
    And a user account "alice@example.com" exists with password "correct-password"

  Scenario: Successful login response sets session cookie with HttpOnly attribute
    When a client POSTs to /auth/login with correct credentials
    Then the HTTP response includes a Set-Cookie header
    And the Set-Cookie header includes the "HttpOnly" attribute

  Scenario: Successful login response sets session cookie with Secure attribute
    When a client POSTs to /auth/login with correct credentials
    Then the Set-Cookie header includes the "Secure" attribute

  Scenario: Successful login response sets session cookie with SameSite=Strict attribute
    When a client POSTs to /auth/login with correct credentials
    Then the Set-Cookie header includes "SameSite=Strict"

  Scenario: Session token value is not present in the login response body
    When a client POSTs to /auth/login with correct credentials
    Then the HTTP response status is 200
    And the response body does not contain the session token string
    And the response body does not contain a field named "token", "session_token", or "access_token"

  Scenario: Session token is not included in any URL in any response
    Given the client is authenticated
    When the client navigates through the application and inspects all response URLs
    Then no URL in any response contains the session token as a path segment or query parameter

  Scenario: Session cookie has Max-Age or Expires attribute matching server-side expiry
    Given the server session expiry is configured to 24 hours
    When a client POSTs to /auth/login with correct credentials
    Then the Set-Cookie header includes a "Max-Age" or "Expires" attribute
    And the specified expiry duration is approximately 24 hours

  Scenario: Expired session results in 401 even if the cookie is structurally valid
    Given a client has a session cookie that was valid but whose server-side session record has expired
    When the client requests a protected endpoint GET /leagues
    Then the HTTP response status is 401
    And the session is not implicitly renewed

  Scenario: Logout clears the session cookie with Max-Age=0
    Given the client is logged in with a valid session cookie
    When the client POSTs to /auth/logout
    Then the HTTP response includes a Set-Cookie header with Max-Age=0 (or equivalent expiry in the past)
    And subsequent requests using the old session cookie return 401

  Scenario: Session tokens are not logged in application logs
    Given the application logging system is inspected after a login event
    When a client logs in and their session is established
    Then the application log files do not contain the session token string
    And the access log does not contain the session token value in request or response headers

  Scenario: JavaScript cannot read the session cookie (HttpOnly enforcement)
    Given a browser client is logged in
    When JavaScript code attempts to read document.cookie
    Then the session cookie value is not accessible to JavaScript
    And document.cookie does not contain the session token name

  Scenario: Session cookie is not transmitted over unencrypted HTTP connections
    Given the Secure cookie attribute is set
    When the client is on an HTTP (not HTTPS) connection and makes a request
    Then the browser does not include the session cookie in the request
    And no session token is transmitted over the unencrypted connection

  Scenario: Secure attribute is suppressed when HTTPS_ENABLED=false (development)
    Given the server is running with HTTPS_ENABLED=false
    When a client POSTs to /auth/login with correct credentials
    Then the HTTP response status is 200
    And the Set-Cookie header includes the "HttpOnly" attribute
    And the Set-Cookie header includes "SameSite=Strict"
    And the Set-Cookie header does NOT include the "Secure" attribute

  Scenario: Secure attribute is present when HTTPS_ENABLED=true (production)
    Given the server is running with HTTPS_ENABLED=true
    When a client POSTs to /auth/login with correct credentials
    Then the Set-Cookie header includes the "HttpOnly" attribute
    And the Set-Cookie header includes the "Secure" attribute
    And the Set-Cookie header includes "SameSite=Strict"
```
