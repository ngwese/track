---- MODULE Common ----
(***************************************************************************
  Shared operators for the Track hub sync TLA+ model (ADR 0004 / ADR 0006).

  Phase 0 (v0): push, pull, acknowledgement promotion, cursor advancement.
  Event authorship is a constant map Author; per-authoring-node cursors are
  introduced in Phase 1.
 ***************************************************************************)
EXTENDS Naturals, Sequences, FiniteSets, TLC

HubLen(log) ==
  Len(log)

AllDurableEvents(hubLog) ==
  {hubLog[i] : i \in DOMAIN hubLog}

AllKnownHubEvents(hubLog, hubAccepted) ==
  AllDurableEvents(hubLog) \union hubAccepted

====
