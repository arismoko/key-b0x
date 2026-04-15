# Releasing

This document is maintainer-facing. It covers the current GitHub Actions
release flow and the manual checks required before publishing a public build.

## Workflows

The repo uses three GitHub Actions workflows:

- `ci.yml`: fast validation on pull requests and pushes to `main`
- `bundle-smoke.yml`: real AppImage and NSIS packaging on `main` and manual
  runs, uploaded as workflow artifacts
- `release.yml`: tag-driven draft GitHub Releases with AppImage, NSIS, and
  `SHA256SUMS.txt`

## Release Steps

1. Bump the desktop app version in `apps/desktop/package.json`.
2. Merge the release candidate to `main`.
3. Let `bundle-smoke.yml` build the real installers from `main`.
4. Manually validate the Linux AppImage and Windows NSIS installer before any
   public release.
5. Create a `vX.Y.Z` tag.
6. Let `release.yml` build the same packaging matrix and create a draft GitHub
   Release.
7. Validate the draft release artifacts and checksums, then publish it.

## Notes

- Public release remains gated on manual installer verification on both Linux
  and Windows.
- Reserve the protected GitHub `release` environment for Windows signing
  secrets before the first public release.
- Linux AppImage releases target modern Linux distributions rather than
  oldest-possible glibc compatibility.
