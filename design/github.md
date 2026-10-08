repo: ShearesWeb/cupid
branch: main
path: ui/src

## Last sync
date: 2026-09-17T12:05:00Z

### Updated in this project
- Replaced appeals with preallocations end-to-end (statuses, seats, quota, detail sections, event timeline).
- Added the Preallocations screen: grant form with applicant/position combos, notes, armed remove.
- Allocations screen now matches remote: main/block/sub type chips, 9-per-page cards, chair-coverage and ballot-coverage meters.
- Review & commit rebuilt on the export flow: per-position hold-back checklist, SSH access check, merge-request export, scoped archive + purge.
- Quota rule ported from cupid-core (main + block ≤ 2, sub ≤ 3, never 1 main + 2 sub); CCA details / home screen intentionally excluded.

## Screen map
| Screen | Repo files |
| --- | --- |
| App shell, sidebar, top bar, toasts | ui/src/App.tsx, ui/src/components/Toasts.tsx |
| Allocations (position + applicant views) | ui/src/screens/Allocations.tsx, ui/src/components/ChairCoverage.tsx, ChoiceCoverage.tsx, CoverageMeter.tsx |
| Preallocations | ui/src/screens/Preallocations.tsx |
| Review & commit | ui/src/screens/Review.tsx, ui/src/lib/selection.ts |
| Applicant / position detail | ui/src/screens/ApplicantDetail.tsx, PositionDetail.tsx, DetailPage.tsx, shared.tsx, ui/src/components/QuotaWidget.tsx, MatchRow.tsx |
| Event history sidebar | ui/src/screens/EventSidebar.tsx |
| Mock engine + snapshot views (cca-data.js) | crates/cupid-core/src/models/capacity.rs, ui/src/lib/types.ts, ui/src/lib/indexes.ts |
| CCAs / home (excluded on request) | ui/src/screens/Ccas.tsx |
