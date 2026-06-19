---- MODULE Snapshots ----
(***************************************************************************
  Phase 3: published snapshot records and snapshot-assisted bootstrap
  (ADR 0004 §Snapshot protocol, §Snapshot-assisted sync).
 ***************************************************************************)
EXTENDS Common

HubLenAbsolute(hubLog, compactedThrough) ==
  compactedThrough + Len(hubLog)

EventAtOffset(hubLog, compactedThrough, absOffset) ==
  hubLog[absOffset - compactedThrough]

AllVisibleEvents(hubLog, compactedThrough, archivedEvents) ==
  archivedEvents \union AllDurableEvents(hubLog)

SnapshotCoverage(hubLog, compactedThrough, archivedEvents, tombstones) ==
  AllVisibleEvents(hubLog, compactedThrough, archivedEvents) \union tombstones

EventAbsoluteOffset(event, hubLog, compactedThrough, archivedOffsets,
                    archivedEvents) ==
  IF event \in AllDurableEvents(hubLog)
  THEN compactedThrough +
         (CHOOSE i \in DOMAIN hubLog : hubLog[i] = event)
  ELSE IF event \in archivedEvents
       THEN archivedOffsets[event]
       ELSE 0

CoverageCursors(snapshotCoverage, hubLog, compactedThrough, archivedOffsets,
                archivedEvents, authorOf, nodesDom) ==
  [a \in nodesDom |->
    LET authorEvents == {e \in snapshotCoverage : authorOf[e] = a}
        offsets ==
          {EventAbsoluteOffset(e, hubLog, compactedThrough, archivedOffsets,
                               archivedEvents) : e \in authorEvents}
    IN IF offsets = {}
       THEN 0
       ELSE CHOOSE off \in offsets : \A off2 \in offsets : off2 <= off]

CanPublishSnapshot(hubLog, compactedThrough, archivedEvents, snapshotThrough) ==
  /\ AllVisibleEvents(hubLog, compactedThrough, archivedEvents) # {}
  /\ HubLenAbsolute(hubLog, compactedThrough) > snapshotThrough

CanBootstrapFromSnapshot(snapshotThrough, bootstrapped, node) ==
  /\ snapshotThrough > 0
  /\ ~bootstrapped[node]

====
