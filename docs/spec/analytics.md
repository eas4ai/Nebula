# Analytics

Prefix: ANALYTICS

Optional, cookieless page analytics through Plausible, off by default. It is on only when both `PLAUSIBLE_SRC` (the script URL on a Plausible instance) and `PLAUSIBLE_DOMAIN` (the site name registered there) are set. The root template `templates/app.html` writes the script tag with the request's CSP nonce, so no finished HTML is rewritten. Plausible's script sends its events to its own origin, which the default Content-Security-Policy blocks (`connect-src 'self'`). Suprnova has no way to add an origin to its preset policy, so when analytics is on Nebula sets the policy itself: the framework's policy for the same `SECURITY_HEADERS` preset, with the Plausible origin added to `connect-src`. Plausible stores nothing on the visitor's device; an app that swaps in a tool that sets cookies takes on the consent question.

The mechanism `analytics` runs the tests named `analytics_<nnn>_…` in `tests/http_flows.rs` and reports each test's result for `ANALYTICS-<nnn>`.

[ANALYTICS-001] When `PLAUSIBLE_SRC` and `PLAUSIBLE_DOMAIN` are both set, every first-visit HTML page MUST load the Plausible script exactly once, as `<script defer data-domain="<PLAUSIBLE_DOMAIN>" src="<PLAUSIBLE_SRC>">` carrying the request's CSP nonce.
Falsifier: With `PLAUSIBLE_SRC=https://plausible.example/js/script.js` and `PLAUSIBLE_DOMAIN=nebula.test`, the first-visit HTML of `/` or `/login` has no such script, has it more than once, or its `nonce` differs from the nonce in the response's `Content-Security-Policy`.
Mechanism: analytics
Status: Agreed 2026-10-16

[ANALYTICS-002] When either variable is unset or empty, Nebula MUST load no analytics script and send the framework's own Content-Security-Policy.
Falsifier: With only `PLAUSIBLE_SRC` set, or only `PLAUSIBLE_DOMAIN`, or neither, the first-visit HTML of `/` has a `script` whose `src` is the `PLAUSIBLE_SRC` value, or the `Content-Security-Policy` of `/`, nonce aside, differs from the one with neither variable set.
Mechanism: analytics
Status: Agreed 2026-10-16

[ANALYTICS-003] When analytics is on, the Content-Security-Policy MUST allow the Plausible script's origin in `connect-src` and otherwise equal the framework's policy for the same `SECURITY_HEADERS` preset.
Falsifier: With both variables set as in ANALYTICS-001, the `connect-src` of the `Content-Security-Policy` of `/` lacks `https://plausible.example`, or, with that origin removed and the nonce aside, the policy differs from the one with analytics off.
Mechanism: analytics
Status: Agreed 2026-10-16

[ANALYTICS-004] The script tag MUST escape `PLAUSIBLE_DOMAIN` and `PLAUSIBLE_SRC`, so a value holding a quote or an angle bracket cannot add an attribute or an element.
Falsifier: With `PLAUSIBLE_DOMAIN` set to `x" onload="alert(1)`, the first-visit HTML of `/` holds the attribute `onload="alert(1)"` on any tag.
Mechanism: analytics
Status: Agreed 2026-10-16

[ANALYTICS-005] Nebula's server MUST refuse to start when `PLAUSIBLE_SRC` is set but is not an absolute `http://` or `https://` URL, with an error that names `PLAUSIBLE_SRC`.
Falsifier: With `PLAUSIBLE_SRC=javascript:alert(1)` and `PLAUSIBLE_DOMAIN=nebula.test`, `try_register_http_stack` returns `Ok`, or an error that does not name `PLAUSIBLE_SRC`.
Mechanism: analytics
Rationale: As `SecurityHeaders::from_env` does for its own settings, a misspelt value fails at boot instead of loading a script from an unexpected place.
Status: Agreed 2026-10-16
