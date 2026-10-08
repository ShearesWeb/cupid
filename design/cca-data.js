/* =========================================================================
   Cupid — mock corpus + matching engine, shaped like the real app's snapshot.
   Mirrors ShearesWeb/cupid @ main:
     - crates/cupid-core/src/models/capacity.rs  (quota rule)
     - ui/src/lib/types.ts                       (snapshot view types)
     - ui/src/lib/indexes.ts                     (derived indexes)

   Model
   -----
   - Committed appointments are the system of record. Chairs only rank
     candidates still to be allocated; applicants only rank positions they
     don't already hold. Committed seats pre-occupy capacity and count toward
     the holder's quota.
   - Preallocations are operator-fixed (applicant, position) pairs. The pair
     is seated BEFORE matching, consumes the seat and the holder's quota, and
     cannot be displaced.
   - Everything else competes for what's left (applicant-proposing deferred
     acceptance over the chair's ranking), under the typed quota rule:
       main + block <= 2, sub <= 3, and never (main >= 1 and sub >= 2).
   ========================================================================= */
(function () {
  "use strict";

  /** Choices the intranet form allows one resident per allocation round. */
  const CHOICES_PER_ROUND = 6;

  // ---- quota rule (port of HeldCounts) --------------------------------
  const emptyHeld = () => ({ main: 0, block: 0, sub: 0 });
  function withinQuota(h) {
    return h.main + h.block <= 2 && h.sub <= 3 && !(h.main >= 1 && h.sub >= 2);
  }
  function canAdd(h, type) {
    const n = { main: h.main, block: h.block, sub: h.sub };
    n[type] += 1;
    return withinQuota(n);
  }

  const ccas = [
    { id: "bball", name: "Basketball", kind: "sports" },
    { id: "choir", name: "Choir", kind: "culture" },
    { id: "jcrc", name: "JCRC", kind: "jcrc" },
    { id: "dance", name: "Dance", kind: "culture" },
  ];

  // chairRank = ordered NEW candidates only. type: block | main | sub.
  const positions = [
    { id: "bball-cap",  ccaId: "bball", name: "Captain",      type: "main",  capacity: 1, chairRank: ["a6", "a2", "a19", "a10", "a13"] },
    { id: "bball-play", ccaId: "bball", name: "Player",       type: "block", capacity: 6, chairRank: ["a6", "a19", "a2", "a10", "a23", "a13", "a17", "a15", "a21", "a25"] },
    { id: "choir-lead", ccaId: "choir", name: "Section Lead", type: "sub",   capacity: 2, chairRank: ["a7", "a18", "a3", "a1", "a14", "a22", "a26"] },
    { id: "choir-mem",  ccaId: "choir", name: "Member",       type: "block", capacity: 8, chairRank: ["a14", "a3", "a26", "a7", "a16", "a20", "a22", "a24", "a1", "a13", "a6", "a9", "a25", "a18", "a5", "a10", "a21", "a17", "a15", "a4"] },
    { id: "jcrc-pres",  ccaId: "jcrc",  name: "President",    type: "main",  capacity: 1, chairRank: ["a8", "a4", "a1", "a19"] },
    { id: "jcrc-welf",  ccaId: "jcrc",  name: "Welfare IC",   type: "sub",   capacity: 2, chairRank: ["a5", "a4", "a16", "a14", "a22"] },
    { id: "dance-cap",  ccaId: "dance", name: "Captain",      type: "main",  capacity: 1, chairRank: ["a5", "a9", "a18", "a2"] },
    { id: "dance-mem",  ccaId: "dance", name: "Member",       type: "block", capacity: 4, chairRank: ["a9", "a24", "a20", "a26", "a13", "a17", "a21", "a7", "a16", "a10"] },
  ];

  // prefs = ranked NEW positions, up to CHOICES_PER_ROUND per applicant.
  const applicants = [
    { id: "a1",  name: "Aishwarya Rajan", email: "aishwarya.r@u.nus.edu",  prefs: ["jcrc-pres", "choir-lead", "choir-mem", "jcrc-welf"] },
    { id: "a2",  name: "Marcus Tan",      email: "marcus.tan@u.nus.edu",   prefs: ["bball-cap", "bball-play"] },
    { id: "a3",  name: "Priya Menon",     email: "priya.menon@u.nus.edu",  prefs: ["choir-mem"] },
    { id: "a4",  name: "Daniel Wong",     email: "daniel.wong@u.nus.edu",  prefs: ["jcrc-pres", "jcrc-welf", "choir-mem"] },
    { id: "a5",  name: "Nurul Huda",      email: "nurul.huda@u.nus.edu",   prefs: ["dance-cap", "jcrc-welf", "choir-mem"] },
    { id: "a6",  name: "Ethan Lim",       email: "ethan.lim@u.nus.edu",    prefs: ["bball-cap", "bball-play", "choir-mem"] },
    { id: "a7",  name: "Sofia Reyes",     email: "sofia.reyes@u.nus.edu",  prefs: ["choir-lead", "choir-mem", "dance-mem"] },
    { id: "a8",  name: "Rahul Iyer",      email: "rahul.iyer@u.nus.edu",   prefs: ["jcrc-pres", "choir-mem"] },
    { id: "a9",  name: "Chloe Goh",       email: "chloe.goh@u.nus.edu",    prefs: ["dance-cap", "dance-mem", "choir-mem"] },
    { id: "a10", name: "Wei Jie Ong",     email: "weijie.ong@u.nus.edu",   prefs: ["bball-cap", "bball-play", "choir-mem", "dance-mem"] },
    { id: "a11", name: "Hannah Lee",      email: "hannah.lee@u.nus.edu",   prefs: ["choir-mem", "dance-mem"] },
    { id: "a12", name: "Mei Ling Chua",   email: "meiling.chua@u.nus.edu", prefs: [] },
    { id: "a13", name: "Arjun Nair",      email: "arjun.nair@u.nus.edu",   prefs: ["bball-play", "choir-mem", "dance-mem", "jcrc-welf", "choir-lead", "dance-cap"] },
    { id: "a14", name: "Grace Toh",       email: "grace.toh@u.nus.edu",    prefs: ["choir-mem", "choir-lead", "jcrc-welf"] },
    { id: "a15", name: "Ryan Chen",       email: "ryan.chen@u.nus.edu",    prefs: ["bball-play", "choir-mem"] },
    { id: "a16", name: "Siti Aminah",     email: "siti.aminah@u.nus.edu",  prefs: ["choir-mem", "jcrc-welf", "dance-mem"] },
    { id: "a17", name: "Lucas Ng",        email: "lucas.ng@u.nus.edu",     prefs: ["bball-play", "dance-mem", "choir-mem"] },
    { id: "a18", name: "Isabelle Koh",    email: "isabelle.koh@u.nus.edu", prefs: ["choir-lead", "choir-mem", "dance-cap"] },
    { id: "a19", name: "Kabir Singh",     email: "kabir.singh@u.nus.edu",  prefs: ["bball-play", "jcrc-pres", "choir-mem"] },
    { id: "a20", name: "Emma Tay",        email: "emma.tay@u.nus.edu",     prefs: ["choir-mem", "dance-mem"] },
    { id: "a21", name: "Jun Kai Ho",      email: "junkai.ho@u.nus.edu",    prefs: ["bball-play", "choir-mem", "dance-mem"] },
    { id: "a22", name: "Vanessa Lim",     email: "vanessa.lim@u.nus.edu",  prefs: ["choir-mem", "choir-lead", "jcrc-welf", "dance-mem", "bball-play", "dance-cap"] },
    { id: "a23", name: "Adam Zulkifli",   email: "adam.z@u.nus.edu",       prefs: ["bball-play", "choir-mem"] },
    { id: "a24", name: "Rachel Foo",      email: "rachel.foo@u.nus.edu",   prefs: ["choir-mem", "dance-mem"] },
    { id: "a25", name: "Dhruv Patel",     email: "dhruv.patel@u.nus.edu",  prefs: ["choir-mem", "bball-play"] },
    { id: "a26", name: "Yuki Tanaka",     email: "yuki.tanaka@u.nus.edu",  prefs: ["choir-mem", "dance-mem", "choir-lead"] },
  ];

  const committed = [
    { applicantId: "a2",  positionId: "choir-mem" },
    { applicantId: "a3",  positionId: "choir-lead" },
    { applicantId: "a11", positionId: "jcrc-welf" },
    { applicantId: "a12", positionId: "dance-mem" },
    { applicantId: "a20", positionId: "bball-play" },
  ];

  // Operator-fixed pairs. Seated before matching; consume seat + quota.
  const preallocations = [
    { applicantId: "a12", positionId: "choir-mem", note: "Chair request — carried over from last cycle." },
    { applicantId: "a5",  positionId: "dance-mem", note: null },
  ];

  const corpus = {
    syncedAt: "2026-06-15T09:42:00",
    warnings: [],
    ccas, positions, applicants, committed, preallocations,
  };

  // ---- matching --------------------------------------------------------
  const pk = (aid, pid) => aid + "|" + pid;

  function runMatching(c) {
    const posById = {}; c.positions.forEach((p) => (posById[p.id] = p));
    const appById = {}; c.applicants.forEach((a) => (appById[a.id] = a));
    const committedSet = new Set(c.committed.map((x) => pk(x.applicantId, x.positionId)));

    const existingByPos = {}; c.positions.forEach((p) => (existingByPos[p.id] = []));
    const held = {}; const ccaHeld = {};
    c.applicants.forEach((a) => { held[a.id] = emptyHeld(); ccaHeld[a.id] = {}; });
    // A user may hold at most one non-resident position per CCA (CapacityStore::can_grant).
    const bump = (aid, pid, d) => {
      const pos = posById[pid];
      held[aid][pos.type] += d;
      ccaHeld[aid][pos.ccaId] = Math.max(0, (ccaHeld[aid][pos.ccaId] || 0) + d);
    };
    const canGrant = (aid, pid) => canAdd(held[aid], posById[pid].type) && !(ccaHeld[aid][posById[pid].ccaId] > 0);
    c.committed.forEach((x) => {
      if (!existingByPos[x.positionId] || !held[x.applicantId]) return;
      existingByPos[x.positionId].push(x.applicantId);
      bump(x.applicantId, x.positionId, 1);
    });

    const preByPos = {}; c.positions.forEach((p) => (preByPos[p.id] = []));
    const newSeats = {}; c.positions.forEach((p) => (newSeats[p.id] = []));
    const rejectedFrom = {}; c.applicants.forEach((a) => (rejectedFrom[a.id] = new Set()));

    const events = [];
    let seq = 0;
    const stamp = (e) => { e.seq = seq++; events.push(e); return e; };

    const chairRankOf = (pid, aid) => { const i = posById[pid].chairRank.indexOf(aid); return i === -1 ? null : i + 1; };
    const prefRankOf = (aid, pid) => { const i = appById[aid].prefs.indexOf(pid); return i === -1 ? null : i + 1; };
    const used = (pid) => existingByPos[pid].length + preByPos[pid].length + newSeats[pid].length;
    const holdsNew = (aid, pid) => newSeats[pid].indexOf(aid) !== -1 || preByPos[pid].indexOf(aid) !== -1;

    // 1. preallocations are seated first and are immovable.
    c.preallocations.forEach((pa) => {
      const pos = posById[pa.positionId];
      if (!pos || !held[pa.applicantId]) return;
      if (committedSet.has(pk(pa.applicantId, pa.positionId))) return;
      if (used(pa.positionId) < pos.capacity) {
        preByPos[pa.positionId].push(pa.applicantId);
        bump(pa.applicantId, pa.positionId, 1);
        stamp({ applicantId: pa.applicantId, positionId: pa.positionId, kind: "accept", reason: "preallocated",
          detail: "Seated by the operator before matching — holds the position outright and cannot be displaced." });
      } else {
        stamp({ applicantId: pa.applicantId, positionId: pa.positionId, kind: "reject", reason: "position-full",
          detail: "Preallocation could not be seated — capacity is already taken." });
      }
    });

    // 2. applicant-proposing deferred acceptance over the remaining seats.
    let guard = 0, progress = true;
    while (progress && guard < 5000) {
      progress = false; guard++;
      for (const app of c.applicants) {
        let target = null;
        for (const pid of app.prefs) {
          if (committedSet.has(pk(app.id, pid))) continue;
          if (holdsNew(app.id, pid)) continue;
          if (rejectedFrom[app.id].has(pid)) continue;
          if (!canGrant(app.id, pid)) continue;
          target = pid; break;
        }
        if (!target) continue;

        const cr = chairRankOf(target, app.id);
        const pr = prefRankOf(app.id, target);
        const open = posById[target].capacity - existingByPos[target].length - preByPos[target].length;

        if (open <= 0) {
          rejectedFrom[app.id].add(target);
          stamp({ applicantId: app.id, positionId: target, kind: "reject", reason: "position-full",
            detail: "No seats to allocate — capacity is taken by existing appointments and preallocations." });
          progress = true; continue;
        }
        if (cr === null) {
          rejectedFrom[app.id].add(target);
          stamp({ applicantId: app.id, positionId: target, kind: "reject", reason: "not-ranked",
            detail: "Chair did not rank this applicant." });
          progress = true; continue;
        }

        const pool = newSeats[target].concat([app.id]);
        pool.sort((x, y) => chairRankOf(target, x) - chairRankOf(target, y));
        const kept = pool.slice(0, open);
        const dropped = pool.slice(open);

        if (kept.indexOf(app.id) !== -1) {
          dropped.forEach((d) => {
            newSeats[target] = newSeats[target].filter((z) => z !== d);
            bump(d, target, -1);
            rejectedFrom[d].add(target);
            stamp({ applicantId: d, positionId: target, kind: "displace", reason: "displaced", byApplicantId: app.id,
              detail: "Displaced by " + appById[app.id].name + " (chair-rank " + cr + ")." });
          });
          newSeats[target].push(app.id);
          newSeats[target].sort((x, y) => chairRankOf(target, x) - chairRankOf(target, y));
          bump(app.id, target, 1);
          stamp({ applicantId: app.id, positionId: target, kind: "accept", reason: "seated",
            detail: "Allocated at chair-rank " + cr + " (their preference #" + pr + ")." });
          progress = true;
        } else {
          rejectedFrom[app.id].add(target);
          const cutoff = chairRankOf(target, kept[kept.length - 1]);
          stamp({ applicantId: app.id, positionId: target, kind: "reject", reason: "position-full",
            detail: "Position full — chair-rank " + cr + " below the cutoff (chair-rank " + cutoff + ")." });
          progress = true;
        }
      }
    }

    // 3. ranked prefs left untaken because the loadout is already full.
    c.applicants.forEach((app) => {
      app.prefs.forEach((pid) => {
        if (committedSet.has(pk(app.id, pid)) || holdsNew(app.id, pid)) return;
        if (events.some((e) => e.applicantId === app.id && e.positionId === pid)) return;
        if (canGrant(app.id, pid)) return;
        if (ccaHeld[app.id][posById[pid].ccaId] > 0) {
          const cca = c.ccas.find((x) => x.id === posById[pid].ccaId);
          stamp({ applicantId: app.id, positionId: pid, kind: "reject", reason: "cca-full",
            detail: "Already holds a position in " + (cca ? cca.name : "this CCA") + " — one position per CCA." });
          return;
        }
        stamp({ applicantId: app.id, positionId: pid, kind: "reject", reason: "quota-full",
          detail: "Quota full — already holding " + loadoutText(held[app.id]) + "; adding a " + posById[pid].type +
            " seat would break the main + block \u2264 2 / sub \u2264 3 rule." });
      });
    });

    const unfilled = [];
    c.positions.forEach((p) => {
      const open = p.capacity - used(p.id);
      if (open > 0) unfilled.push({ positionId: p.id, open });
    });

    const assignments = [];
    c.positions.forEach((p) => {
      newSeats[p.id].forEach((aid) => assignments.push({ applicantId: aid, positionId: p.id, kind: "allocated",
        chairRank: chairRankOf(p.id, aid), prefRank: prefRankOf(aid, p.id) }));
      preByPos[p.id].forEach((aid) => assignments.push({ applicantId: aid, positionId: p.id, kind: "preallocated",
        chairRank: chairRankOf(p.id, aid), prefRank: prefRankOf(aid, p.id) }));
    });

    return { assignments, events, unfilled };
  }

  function loadoutText(h) {
    const parts = [];
    if (h.main) parts.push(h.main + " main");
    if (h.block) parts.push(h.block + " block");
    if (h.sub) parts.push(h.sub + " sub");
    return parts.length ? parts.join(" + ") : "nothing";
  }

  // ---- snapshot (seats / quota / outcomes, as the backend serves them) ---
  function buildSnapshot(c, run) {
    const posById = {}; c.positions.forEach((p) => (posById[p.id] = p));
    const appById = {}; c.applicants.forEach((a) => (appById[a.id] = a));
    const committedSet = new Set(c.committed.map((x) => pk(x.applicantId, x.positionId)));
    const assignments = run ? run.assignments : [];
    const events = run ? run.events : [];

    const seats = c.positions.map((p) => {
      const seated = [];
      c.committed.forEach((x) => { if (x.positionId === p.id) seated.push({ applicantId: x.applicantId, status: "existing" }); });
      if (run) {
        assignments.forEach((a) => { if (a.positionId === p.id && a.kind === "allocated" && !committedSet.has(pk(a.applicantId, a.positionId))) seated.push({ applicantId: a.applicantId, status: "allocated" }); });
        assignments.forEach((a) => { if (a.positionId === p.id && a.kind === "preallocated" && !committedSet.has(pk(a.applicantId, a.positionId))) seated.push({ applicantId: a.applicantId, status: "preallocated" }); });
      } else {
        // Pre-run the preallocation is already the claim on the seat.
        c.preallocations.forEach((pa) => { if (pa.positionId === p.id && !committedSet.has(pk(pa.applicantId, pa.positionId))) seated.push({ applicantId: pa.applicantId, status: "preallocated" }); });
      }
      return { positionId: p.id, seated };
    });

    const heldBy = {}; c.applicants.forEach((a) => (heldBy[a.id] = emptyHeld()));
    seats.forEach((sv) => {
      const type = posById[sv.positionId].type;
      sv.seated.forEach((s) => { if (heldBy[s.applicantId]) heldBy[s.applicantId][type] += 1; });
    });
    const quota = c.applicants.map((a) => {
      const h = heldBy[a.id];
      return { applicantId: a.id, main: h.main, block: h.block, sub: h.sub,
        canAddMain: canAdd(h, "main"), canAddBlock: canAdd(h, "block"), canAddSub: canAdd(h, "sub"),
        over: !withinQuota(h) };
    });

    // statuses for every seat, plus an outcome for every ranked pair.
    const statusByPair = {};
    seats.forEach((sv) => sv.seated.forEach((s) => { statusByPair[pk(s.applicantId, sv.positionId)] = s.status; }));
    const lastEvent = {};
    events.forEach((e) => { lastEvent[pk(e.applicantId, e.positionId)] = e; });

    const pairs = new Set(Object.keys(statusByPair));
    c.applicants.forEach((a) => a.prefs.forEach((pid) => pairs.add(pk(a.id, pid))));
    c.positions.forEach((p) => p.chairRank.forEach((aid) => pairs.add(pk(aid, p.id))));
    c.preallocations.forEach((pa) => pairs.add(pk(pa.applicantId, pa.positionId)));

    const outcomes = [];
    pairs.forEach((key) => {
      const [aid, pid] = key.split("|");
      if (!appById[aid] || !posById[pid]) return;
      outcomes.push(Object.assign({ applicantId: aid, positionId: pid }, outcomeFor(c, appById, posById, statusByPair, lastEvent, !!run, aid, pid)));
    });

    return {
      syncedAt: c.syncedAt, warnings: c.warnings,
      ccas: c.ccas, positions: c.positions, applicants: c.applicants,
      committed: c.committed, preallocations: c.preallocations,
      quota, seats, outcomes, run: run || null,
    };
  }

  function outcomeFor(c, appById, posById, statusByPair, lastEvent, hasRun, aid, pid) {
    const key = pk(aid, pid);
    const seated = statusByPair[key];
    if (seated === "existing") return { status: "existing", label: "Appointment", detail: "Already a committed appointment." };
    if (seated === "preallocated") return { status: "preallocated", label: "Preallocated", detail: "Operator-fixed — seated before matching, cannot be displaced." };
    if (seated === "allocated") return { status: "allocated", label: "Allocated", detail: "Newly allocated by this run." };

    const ev = lastEvent[key];
    if (ev) {
      if (ev.reason === "not-ranked") return { status: "neutral", label: "Not ranked", detail: ev.detail };
      if (ev.reason === "position-full") return { status: "neutral", label: "Position full", detail: ev.detail };
      if (ev.reason === "quota-full") return { status: "quota", label: "Quota full", detail: ev.detail };
      if (ev.reason === "cca-full") return { status: "quota", label: "One per CCA", detail: ev.detail };
      if (ev.reason === "displaced") return { status: "displaced", label: "Displaced", detail: ev.detail };
    }
    const a = appById[aid];
    if (a.prefs.indexOf(pid) === -1) {
      const full = a.prefs.length >= CHOICES_PER_ROUND;
      return { status: "noreturn", label: "Didn't rank back", detail: full
        ? "All " + CHOICES_PER_ROUND + " choices used elsewhere."
        : "Didn't list this position (" + a.prefs.length + "/" + CHOICES_PER_ROUND + " choices used)." };
    }
    const pre = c.preallocations.some((p) => p.applicantId === aid && p.positionId === pid);
    if (pre && !hasRun) return { status: "preallocated", label: "Pending", detail: "Granted — re-run matching to seat it." };
    return { status: "neutral", label: hasRun ? "Not allocated" : "\u2014",
      detail: hasRun ? "Not allocated in this run." : "Run matching to see the outcome." };
  }

  window.CUPID = { corpus, runMatching, buildSnapshot, withinQuota, canAdd, loadoutText, CHOICES_PER_ROUND, pk };
})();
