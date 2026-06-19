---- MODULE Network ----
(***************************************************************************
  Phase 2: push/pull stream helpers for partial failure (ADR 0004 §Partial
  failure semantics).
 ***************************************************************************)
EXTENDS Common, Sequences

RECURSIVE TakeFirstEvents(_, _)

TakeFirstEvents(seq, maxCount) ==
  IF maxCount = 0 \/ seq = <<>>
  THEN <<>>
  ELSE <<Head(seq)>> \o TakeFirstEvents(Tail(seq), maxCount - 1)

StreamRemainder(seq, durableLen) ==
  IF durableLen >= Len(seq)
  THEN <<>>
  ELSE SubSeq(seq, durableLen + 1, Len(seq))

PrependSeq(prefix, suffix) == prefix \o suffix

RemainderAfterTake(seq, taken) ==
  IF taken >= Len(seq)
  THEN <<>>
  ELSE SubSeq(seq, taken + 1, Len(seq))

StreamIdle(stream, durableLen) ==
  /\ stream = <<>>
  /\ durableLen = 0

====
