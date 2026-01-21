# Releasing Good Commit

Use this checklist to ship a new version with working Homebrew + npm updates.

## 1) Bump versions

- `crates/core/Cargo.toml`
- `crates/cli/Cargo.toml`
- `npm/package.json`
- `homebrew/goodcommit.rb` (version + URLs; checksums are filled by CI)

## 2) Commit and tag

```bash
git commit -am "chore(release): vX.Y.Z"
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin main vX.Y.Z
```

## 3) CI publishes artifacts

- `release` workflow builds binaries and creates the GitHub release.
- `publish-npm` runs after `release` and publishes `npm/package.json` (must match tag).
- `publish-brew` runs on the GitHub release and updates the tap with checksums.

If a workflow didn't run, trigger it manually:

- `release`: push the tag again or re-run in GitHub Actions.
- `publish-npm`: run workflow with the tag.
- `publish-brew`: run workflow with the tag.

## 4) Verify installs

```bash
brew upgrade goodcommit
goodcommit --version
npm view goodcommit version
```
