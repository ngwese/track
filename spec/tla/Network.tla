---- MODULE Network ----
(***************************************************************************
  Phase 2: nondeterministic message delay, duplication, loss, and abort.

  v0 models push/pull as atomic logical steps in HubSync.tla. This module will
  split those steps across network channels per ADR 0004 §Partial failure
  semantics.
 ***************************************************************************)
====
