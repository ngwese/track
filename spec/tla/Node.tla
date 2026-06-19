---- MODULE Node ----
(***************************************************************************
  Replica-side transitions: pull delivery, local persist, cursor advance.

  Phase 1: per-authoring-node cursors (ADR 0004 §Cursor model).
  Phase 3–4: absolute hub offsets with compacted prefix (ADR 0004 §Compaction).
 ***************************************************************************)
EXTENDS Common, Snapshots

RECURSIVE CollectPullPage(_, _, _, _, _, _, _)

CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, compactedThrough,
                nextAbsIdx, acc) ==
  LET absMax == HubLenAbsolute(hubLog, compactedThrough)
  IN IF Len(acc) >= pageLimit \/ nextAbsIdx > absMax
     THEN acc
     ELSE LET event == EventAtOffset(hubLog, compactedThrough, nextAbsIdx)
              author == authorOf[event]
          IN IF nextAbsIdx > cursorsNode[author]
             THEN CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf,
                                  compactedThrough, nextAbsIdx + 1,
                                  Append(acc, event))
             ELSE CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf,
                                  compactedThrough, nextAbsIdx + 1, acc)

PullWindow(hubLog, cursorsNode, pageLimit, authorOf, compactedThrough) ==
  IF compactedThrough >= HubLenAbsolute(hubLog, compactedThrough)
  THEN <<>>
  ELSE CollectPullPage(hubLog, cursorsNode, pageLimit, authorOf, compactedThrough,
                       compactedThrough + 1, <<>>)

HubOffsetOfEvent(hubLog, event, compactedThrough) ==
  compactedThrough +
    (CHOOSE i \in DOMAIN hubLog : hubLog[i] = event)

PersistBeforeCursorOK(hubLog, localEvents, cursorsNode, authorOf,
                      compactedThrough, archivedEvents, archivedOffsets) ==
  LET absMax == HubLenAbsolute(hubLog, compactedThrough)
  IN /\ IF compactedThrough >= absMax
        THEN TRUE
        ELSE \A author \in DOMAIN cursorsNode :
               \A absIdx \in (compactedThrough + 1)..absMax :
                 (authorOf[EventAtOffset(hubLog, compactedThrough, absIdx)] = author
                  /\ absIdx <= cursorsNode[author])
                   => EventAtOffset(hubLog, compactedThrough, absIdx) \in localEvents
     /\ \A author \in DOMAIN cursorsNode :
          \A e \in archivedEvents :
            (authorOf[e] = author
             /\ archivedOffsets[e] <= cursorsNode[author])
              => e \in localEvents

MinUnseenOffset(hubLog, cursorsNode, authorOf, compactedThrough) ==
  LET absMax == HubLenAbsolute(hubLog, compactedThrough)
  IN IF compactedThrough >= absMax
     THEN 0
     ELSE IF \E absIdx \in (compactedThrough + 1)..absMax :
              absIdx > cursorsNode[authorOf[EventAtOffset(hubLog, compactedThrough,
                                                          absIdx)]]
          THEN CHOOSE absIdx \in (compactedThrough + 1)..absMax :
                 /\ absIdx > cursorsNode[authorOf[EventAtOffset(hubLog,
                                                                 compactedThrough,
                                                                 absIdx)]]
                 /\ \A j \in (compactedThrough + 1)..absMax :
                      (j < absIdx)
                        => (j <= cursorsNode[authorOf[EventAtOffset(hubLog,
                                                                      compactedThrough,
                                                                      j)]])
          ELSE 0

====
