---- MODULE Node ----
(***************************************************************************
  Replica-side transitions: pull delivery, local persist, cursor advance.

  Phase 1: per-authoring-node cursors (ADR 0004 §Cursor model).
 ***************************************************************************)
EXTENDS Common

RECURSIVE CollectPullPage(_, _, _, _, _, _)

CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, nextIdx, acc) ==
  IF Len(acc) >= pageLimit \/ nextIdx > HubLen(hubLog)
  THEN acc
  ELSE IF nextIdx > cursorsNode[authorOf[hubLog[nextIdx]]]
       THEN CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, nextIdx + 1,
                            Append(acc, hubLog[nextIdx]))
       ELSE CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, nextIdx + 1, acc)

PullWindow(hubLog, cursorsNode, pageLimit, authorOf) ==
  CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, 1, <<>>)

NextOffsetForAuthor(hubLog, author, cursor, authorOf) ==
  IF \E i \in DOMAIN hubLog : authorOf[hubLog[i]] = author /\ i > cursor
  THEN CHOOSE i \in DOMAIN hubLog :
         /\ authorOf[hubLog[i]] = author
         /\ i > cursor
         /\ \A j \in DOMAIN hubLog :
              (authorOf[hubLog[j]] = author /\ j < i /\ j > cursor) => FALSE
  ELSE 0

PersistBeforeCursorOK(hubLog, localEvents, cursorsNode, authorOf) ==
  \A author \in DOMAIN cursorsNode :
    \A i \in DOMAIN hubLog :
      (authorOf[hubLog[i]] = author /\ i <= cursorsNode[author])
        => hubLog[i] \in localEvents

MinUnseenOffset(hubLog, cursorsNode, authorOf) ==
  IF \E i \in DOMAIN hubLog : i > cursorsNode[authorOf[hubLog[i]]]
  THEN CHOOSE i \in DOMAIN hubLog :
         /\ i > cursorsNode[authorOf[hubLog[i]]]
         /\ \A j \in DOMAIN hubLog :
              (j < i) => (j <= cursorsNode[authorOf[hubLog[j]]])
  ELSE 0

HubOffsetOfEvent(hubLog, event) ==
  CHOOSE i \in DOMAIN hubLog : hubLog[i] = event

====
