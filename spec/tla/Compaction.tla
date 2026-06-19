---- MODULE Compaction ----
(***************************************************************************
  Phase 4: compaction watermarks, inactive replica policy, tombstone retention
  (ADR 0004 §Compaction and retention).
 ***************************************************************************)
EXTENDS Common, Snapshots

MinReplicaWatermark(cursorsNode) ==
  LET reported == {cursorsNode[a] : a \in DOMAIN cursorsNode}
  IN IF reported = {}
     THEN 0
     ELSE CHOOSE w \in reported : \A w2 \in reported : w <= w2

PrefixEventsRemoved(hubLog, compactedThrough, boundary) ==
  {hubLog[i] : i \in 1..(boundary - compactedThrough)}

ArchiveOffsets(allEvents, hubLog, compactedThrough, boundary, archivedOffsets) ==
  LET removed == PrefixEventsRemoved(hubLog, compactedThrough, boundary)
  IN [e \in allEvents |->
        IF e \in removed
        THEN compactedThrough +
               (CHOOSE i \in 1..(boundary - compactedThrough) : hubLog[i] = e)
        ELSE archivedOffsets[e]]

CompactHubLog(hubLog, compactedThrough, boundary) ==
  SubSeq(hubLog, boundary - compactedThrough + 1, Len(hubLog))

CanReportWatermark(cursorsNode, localEvents, archivedEvents, compactedThrough) ==
  LET w == MinReplicaWatermark(cursorsNode)
  IN (w < compactedThrough) \/ (archivedEvents \subseteq localEvents)

CanCompactThrough(hubLog, compactedThrough, boundary, snapshotThrough,
                  snapshotCoverage, tombstones, replicaWatermark, nodeActive,
                  nodesDom, archivedEvents, localLog) ==
  /\ boundary > compactedThrough
  /\ boundary <= snapshotThrough
  /\ tombstones \subseteq snapshotCoverage
  /\ \A n \in nodesDom :
       (~nodeActive[n]) \/ (replicaWatermark[n] >= boundary)
  /\ LET toRemove == PrefixEventsRemoved(hubLog, compactedThrough, boundary)
     IN \A n \in nodesDom :
          nodeActive[n] /\ replicaWatermark[n] >= boundary
            => toRemove \subseteq localLog[n]
  /\ PrefixEventsRemoved(hubLog, compactedThrough, boundary)
       \subseteq snapshotCoverage

====
