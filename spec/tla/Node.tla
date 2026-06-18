---- MODULE Node ----
(***************************************************************************
  Replica-side transitions: pull delivery, local persist, cursor advance.
 ***************************************************************************)
EXTENDS Common

PullWindow(hubLog, cursor, pageLimit) ==
  LET hubLen == HubLen(hubLog)
      start == cursor + 1
      last ==
        IF hubLen = 0
        THEN 0
        ELSE IF start + pageLimit - 1 <= hubLen
             THEN start + pageLimit - 1
             ELSE hubLen
  IN IF start > hubLen \/ start > last
     THEN <<>>
     ELSE SubSeq(hubLog, start, last)

PersistBeforeCursorOK(hubLog, localEvents, cursor) ==
  IF cursor = 0
  THEN TRUE
  ELSE \A i \in 1..cursor : hubLog[i] \in localEvents

====
