> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Release Governance / Changelog Evidence
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Historical release headings can be overwritten by synchronization tools.
> - **Observability**: Enforced by `scripts/check_changelog.cjs`.

# Changelog reconstruction evidence

The legacy synchronization script rewrote every released heading in `CHANGELOG.md` to the current product version. The headings were reconstructed on 2026-08-24 from the private repository history using the first commit that introduced each section.

| Release section | Restored version | Evidence |
| --- | ---: | --- |
| 2026-07-22 | 1.1.281 | Commit `1f88158d` introduced the section with subject `docs(changelog): update 1.1.281 release notes`. Its committed `version.json` had already advanced to 1.1.282, so the release-note subject is the least ambiguous evidence of intent. |
| 2026-05-24 | 1.1.96 | Commit `44feae61` first introduced the section while `version.json` was 1.1.96. |
| 2026-04-16 | 1.1.6 | Commit `7adf8693` introduced both adjacent 1.1.6 entries; they were merged into one release section without changing their entries. |

This reconstruction does not claim that every intermediate historical version had a formal release. From this point forward, published sections are immutable and a product-version transition must introduce exactly one matching release heading.
