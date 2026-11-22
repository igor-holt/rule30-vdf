# Releasing rule30-vdf

Releases go to [crates.io](https://crates.io/crates/rule30-vdf) from `.github/workflows/release.yml`. The workflow runs when a tag such as `v0.1.0` is pushed. It publishes through crates.io Trusted Publishing, so this repository stores no crates.io API token. The only credential involved is a short-lived token that the workflow requests while it runs and revokes when the job ends.

A published version is permanent. crates.io does not let anyone overwrite a version, and `cargo yank` only hides a version from new projects. Treat the crate name and every release as something you cannot take back.

## One-time setup

You need the crate owner's crates.io account and admin rights on the GitHub repository.

1. **Publish the first version by hand.** crates.io can attach a trusted publisher only to a crate that already exists.
   - Create an API token at <https://crates.io/settings/tokens> with the `publish-new` scope and a short expiry. If the form offers a crate-name restriction, set it to `rule30-vdf`.
   - From a clean checkout of the commit you want to release, run `cargo publish --dry-run --locked` and read the file list it prints. Then run `cargo publish --locked` with the token (`cargo login`, or the `CARGO_REGISTRY_TOKEN` environment variable).
   - Revoke the token on the same settings page.
2. **Add the trusted publisher** in the crate's settings on crates.io. Choose GitHub Actions and enter these four values. They must match `release.yml` and the GitHub environment exactly.

   | Field | Value |
   |-------|-------|
   | Repository owner | `igor-holt` |
   | Repository name | `rule30-vdf` |
   | Workflow filename | `release.yml` |
   | Environment | `release` |

3. **Create the `release` environment** in the GitHub repository (Settings, Environments). Add yourself as a required reviewer so that every publish waits for your approval. You can also limit the environment to tags that match `v*`.
4. **Optional: enforce Trusted Publishing** in the crate's settings on crates.io. Token-based publishing is then switched off for the crate.
5. **Tag the first release** once the steps above are done: `git tag v0.1.0 && git push origin v0.1.0`. The version is already on crates.io after step 1. The workflow finds it in the index, runs its checks and skips the publish job.

## Each release

1. Pick the version. Set `version` in `Cargo.toml` and in `CITATION.cff` (the workflow stops if either differs from the tag). Run `cargo check` so that `Cargo.lock` carries the new version, and commit the three files.
2. Check locally:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --release --locked
   cargo publish --dry-run --locked
   ```

3. Merge to `main`, then tag the merged commit and push the tag: `git tag v0.1.1 && git push origin v0.1.1`.
4. The **Release** workflow runs. Its `verify` job checks that the tag, `Cargo.toml` and `CITATION.cff` agree, runs the tests, builds the package and looks the version up in the crates.io index. Its `publish` job then waits for approval of the `release` environment, exchanges the job's GitHub identity for a short-lived crates.io token and runs `cargo publish --locked`.
5. When the run finishes, check <https://crates.io/crates/rule30-vdf> and <https://docs.rs/rule30-vdf>.

## If something goes wrong

- **`verify` fails on a version mismatch, or a test fails.** Nothing was published. Fix the problem, delete the tag (`git tag -d vX.Y.Z && git push --delete origin vX.Y.Z`) and tag the corrected commit.
- **`publish` fails after you approved it.** Re-run the failed job from the Actions page. If the version did reach crates.io, the re-run skips the publish step.
- **A bad version is published.** Yank it with `cargo yank --version X.Y.Z` or from the crate's page on crates.io, then bump the version and release the fix.

## Sources

- RFC 3691, Trusted Publishing for crates.io: <https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html>
- crates.io development update, July 2025: <https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/>
- crates.io development update, January 2026 (enforcing Trusted Publishing; `pull_request_target` and `workflow_run` triggers are blocked): <https://blog.rust-lang.org/2026/01/21/crates-io-development-update/>
- The token exchange action used by `release.yml`: <https://github.com/rust-lang/crates-io-auth-action>

Where the crates.io settings pages differ from this guide, the crates.io documentation wins.
