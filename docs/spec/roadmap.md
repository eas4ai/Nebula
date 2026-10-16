# Roadmap

Current: markdown-twins

## markdown-twins

Requirements: TWIN-001, TWIN-002, TWIN-003, TWIN-004, TWIN-005, TWIN-006, TWIN-007, TWIN-008, TWIN-009, TWIN-010, TWIN-011, TWIN-012, TWIN-013, TWIN-014

Delivery: on the `sudus` branch. Add `suprnova-markdown` at `tag = "v0.3.0"`; `Cargo.lock` gains only that package. Add `src/markdown.rs`, holding the public page list with Home, the `PublicPages` source, the `LlmsTxt` middleware, the `SUMMARY` constant and `alternate(path)`. Add `home::content()`, which both the Home handler and the twin use. Register both middlewares after `LoggingMiddleware` and before `SessionMiddleware`. The Home handler adds the alternate link through `Head::try_update`. The `twin_<nnn>_…` tests in `tests/http_flows.rs` are written before the start and fail until this work makes them pass. In `README.md`, add a "Markdown twins for crawlers" section: what twins and `/llms.txt` are, how to add a page to the list, the four safety rules, and the time limit a database-backed source must set itself, since twins run outside `TimeoutMiddleware`. In `UPGRADE.md`, add a 3.0.0 bullet for the new dependency, the middleware position and the new paths.

Done when: every requirement above has a current pass receipt. `cargo test --locked`, `cargo clippy`, `npm run check`, `npm run build` and `npm run build:ssr` pass. In a browser, Home's `<head>` has the alternate link once after a server-rendered visit and after a client-side visit, and the link is gone after navigating to `/login`. `README.md` and `UPGRADE.md` are updated.
