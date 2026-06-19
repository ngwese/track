---- MODULE HubSync ----
(***************************************************************************
  Track hub sync protocol — root specification (ADR 0004 / ADR 0006).

  Phase 4 models:
    - idempotent push with streaming commit and mid-stream abort
    - per-authoring-node cursors and incremental pull delivery
    - published snapshots and snapshot-assisted bootstrap
    - compaction below snapshot and replica watermarks
    - tombstone retention across compaction
 ***************************************************************************)
EXTENDS Common, Hub, Node, Network, Snapshots, Compaction

CONSTANTS Nodes, Events, MaxHubLen, PageLimit, MaxPushStream, TombstoneEvents

\* Authorship for the finite CI model (node 1 authors events 1 and 2).
Author ==
  [event \in Events |-> IF event \in {1, 2} THEN 1 ELSE 2]

ZeroCursors ==
  [n \in Nodes |-> [a \in Nodes |-> 0]]

VARIABLES hubLog, hubAccepted, localLog, cursors, outQueue,
          pushStream, pushDurableLen, pullBuf, pullPending,
          compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
          snapshotCoverage,
          tombstones, replicaWatermark, nodeActive, bootstrapped,
          bootstrapCoverage, snapshotCursors

vars ==
  <<hubLog, hubAccepted, localLog, cursors, outQueue,
    pushStream, pushDurableLen, pullBuf, pullPending,
    compactedThrough, archivedEvents, archivedOffsets, snapshotThrough, snapshotCoverage,
    tombstones, replicaWatermark, nodeActive, bootstrapped,
    bootstrapCoverage, snapshotCursors>>

TypeOK ==
  /\ hubLog \in Seq(Events)
  /\ hubAccepted \subseteq Events
  /\ Len(hubLog) + compactedThrough <= MaxHubLen
  /\ compactedThrough \in Nat
  /\ archivedEvents \subseteq Events
  /\ archivedOffsets \in [Events -> Nat]
  /\ \A e \in archivedEvents :
       /\ archivedOffsets[e] \in 1..compactedThrough
       /\ archivedOffsets[e] > 0
  /\ snapshotThrough \in Nat
  /\ snapshotCoverage \subseteq Events
  /\ tombstones \subseteq Events
  /\ tombstones \subseteq TombstoneEvents
  /\ replicaWatermark \in [Nodes -> Nat]
  /\ nodeActive \in [Nodes -> BOOLEAN]
  /\ bootstrapped \in [Nodes -> BOOLEAN]
  /\ bootstrapCoverage \in [Nodes -> SUBSET Events]
  /\ snapshotCursors \in [Nodes -> Nat]
  /\ localLog \in [Nodes -> SUBSET Events]
  /\ cursors \in [Nodes -> [Nodes -> Nat]]
  /\ \A n \in Nodes, a \in Nodes :
       cursors[n][a] <= HubLenAbsolute(hubLog, compactedThrough)
  /\ compactedThrough <= snapshotThrough
  /\ archivedEvents \subseteq snapshotCoverage
  /\ outQueue \in [Nodes -> Seq(Events)]
  /\ pushStream \in [Nodes -> Seq(Events)]
  /\ pushDurableLen \in [Nodes -> Nat]
  /\ pullBuf \in [Nodes -> Seq(Events)]
  /\ pullPending \in [Nodes -> Seq(Events)]
  /\ \A n \in Nodes :
       pushDurableLen[n] <= Len(pushStream[n])
  /\ \A n \in Nodes :
       \A i \in DOMAIN pullBuf[n] :
         pullBuf[n][i] \in AllDurableEvents(hubLog)

Init ==
  /\ hubLog = <<>>
  /\ hubAccepted = {}
  /\ localLog = [n \in Nodes |-> {}]
  /\ cursors = ZeroCursors
  /\ outQueue = [n \in Nodes |-> <<>>]
  /\ pushStream = [n \in Nodes |-> <<>>]
  /\ pushDurableLen = [n \in Nodes |-> 0]
  /\ pullBuf = [n \in Nodes |-> <<>>]
  /\ pullPending = [n \in Nodes |-> <<>>]
  /\ compactedThrough = 0
  /\ archivedEvents = {}
  /\ archivedOffsets = [e \in Events |-> 0]
  /\ snapshotThrough = 0
  /\ snapshotCoverage = {}
  /\ tombstones = {}
  /\ replicaWatermark = [n \in Nodes |-> 0]
  /\ nodeActive = [n \in Nodes |-> TRUE]
  /\ bootstrapped = [n \in Nodes |-> FALSE]
  /\ bootstrapCoverage = [n \in Nodes |-> {}]
  /\ snapshotCursors = [n \in Nodes |-> 0]

CanSync(node) == nodeActive[node]

CanEnqueue(node, event) ==
  /\ CanSync(node)
  /\ Author[event] = node
  /\ event \notin AllDurableEvents(hubLog)
  /\ event \notin archivedEvents
  /\ event \notin {outQueue[node][i] : i \in DOMAIN outQueue[node]}
  /\ event \notin {pushStream[node][i] : i \in DOMAIN pushStream[node]}

Enqueue(node, event) ==
  /\ CanEnqueue(node, event)
  /\ outQueue' = [outQueue EXCEPT ![node] = Append(@, event)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors,
                 pushStream, pushDurableLen, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

CanStartPush(node) ==
  /\ CanSync(node)
  /\ StreamIdle(pushStream[node], pushDurableLen[node])
  /\ outQueue[node] # <<>>
  /\ HubLenAbsolute(hubLog, compactedThrough) + Cardinality(hubAccepted)
       < MaxHubLen

StartPush(node) ==
  /\ CanStartPush(node)
  /\ LET batch == TakeFirstEvents(outQueue[node], MaxPushStream)
         taken == Len(batch)
     IN /\ pushStream' = [pushStream EXCEPT ![node] = batch]
        /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = 0]
        /\ outQueue' = [outQueue EXCEPT ![node] =
             RemainderAfterTake(@, taken)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

CanPushCommitNext(node) ==
  /\ CanSync(node)
  /\ pushStream[node] # <<>>
  /\ pushDurableLen[node] < Len(pushStream[node])
  /\ HubLenAbsolute(hubLog, compactedThrough) + Cardinality(hubAccepted)
       < MaxHubLen

PushCommitNext(node) ==
  /\ CanPushCommitNext(node)
  /\ LET idx == pushDurableLen[node] + 1
         event == pushStream[node][idx]
         committed == HubCommitEvent(hubLog, hubAccepted, event)
     IN /\ event \in AllDurableEvents(committed.hubLogNew)
        /\ hubLog' = committed.hubLogNew
        /\ hubAccepted' = committed.hubAcceptedNew
        /\ IF idx = Len(pushStream[node])
           THEN /\ pushStream' = [pushStream EXCEPT ![node] = <<>>]
                /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = 0]
           ELSE /\ pushStream' = pushStream
                /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = idx]
  /\ UNCHANGED <<localLog, cursors, outQueue, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

CanAbortPush(node) ==
  /\ CanSync(node)
  /\ pushStream[node] # <<>>
  /\ pushDurableLen[node] < Len(pushStream[node])

AbortPushStream(node) ==
  /\ CanAbortPush(node)
  /\ LET remainder == StreamRemainder(pushStream[node], pushDurableLen[node])
     IN /\ outQueue' = [outQueue EXCEPT ![node] = PrependSeq(remainder, @)]
        /\ pushStream' = [pushStream EXCEPT ![node] = <<>>]
        /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = 0]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

MalformedPush(node) == AbortPushStream(node)
InterruptPush(node) == AbortPushStream(node)

CanBeginPull(node) ==
  /\ CanSync(node)
  /\ pullBuf[node] = <<>>
  /\ pullPending[node] = <<>>
  /\ PullWindow(hubLog, cursors[node], PageLimit, Author, compactedThrough)
       # <<>>

BeginPull(node) ==
  /\ CanBeginPull(node)
  /\ pullPending' = [pullPending EXCEPT ![node] =
       PullWindow(hubLog, cursors[node], PageLimit, Author, compactedThrough)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen, pullBuf,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

CanPullSendNext(node) ==
  /\ CanSync(node)
  /\ pullPending[node] # <<>>

PullSendNext(node) ==
  /\ CanPullSendNext(node)
  /\ LET event == Head(pullPending[node])
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Append(@, event)]
        /\ pullPending' = [pullPending EXCEPT ![node] = Tail(@)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

CanAbortPull(node) ==
  /\ CanSync(node)
  /\ (pullPending[node] # <<>> \/ pullBuf[node] # <<>>)

AbortPullStream(node) ==
  /\ CanAbortPull(node)
  /\ pullPending' = [pullPending EXCEPT ![node] = <<>>]
  /\ pullBuf' = [pullBuf EXCEPT ![node] = <<>>]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark,
                 nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

MalformedPull(node) == AbortPullStream(node)
InterruptPull(node) == AbortPullStream(node)

CanPersist(node) ==
  /\ CanSync(node)
  /\ pullBuf[node] # <<>>

Persist(node) ==
  /\ CanPersist(node)
  /\ LET event == Head(pullBuf[node])
         offset == HubOffsetOfEvent(hubLog, event, compactedThrough)
         author == Author[event]
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Tail(@)]
        /\ localLog' = [localLog EXCEPT ![node] = @ \union {event}]
        /\ cursors' = [cursors EXCEPT ![node][author] = offset]
        /\ tombstones' = IF event \in TombstoneEvents
                         THEN tombstones \union {event}
                         ELSE tombstones
        /\ UNCHANGED <<hubLog, hubAccepted, outQueue,
                       pushStream, pushDurableLen, pullPending,
                       compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                       snapshotCoverage, replicaWatermark,
                       nodeActive, bootstrapped, bootstrapCoverage, snapshotCursors>>

PublishSnapshot ==
  /\ LET coverage ==
       SnapshotCoverage(hubLog, compactedThrough, archivedEvents, tombstones)
         absThrough == HubLenAbsolute(hubLog, compactedThrough)
     IN /\ CanPublishSnapshot(hubLog, compactedThrough, archivedEvents, snapshotThrough)
        /\ snapshotThrough' = absThrough
        /\ snapshotCoverage' = coverage
        /\ snapshotCursors' =
             CoverageCursors(coverage, hubLog, compactedThrough, archivedOffsets,
                             archivedEvents, Author, Nodes)
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, tombstones,
                 replicaWatermark, nodeActive, bootstrapped, bootstrapCoverage>>

ReportWatermark(node) ==
  /\ CanSync(node)
  /\ CanReportWatermark(cursors[node], localLog[node], archivedEvents,
                        compactedThrough)
  /\ replicaWatermark' =
       [replicaWatermark EXCEPT ![node] = MinReplicaWatermark(cursors[node])]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen, pullBuf, pullPending,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, nodeActive, bootstrapped,
                 bootstrapCoverage, snapshotCursors>>

CanCompactPrefix(boundary) ==
  /\ \A n \in Nodes :
       /\ StreamIdle(pushStream[n], pushDurableLen[n])
       /\ pullBuf[n] = <<>>
       /\ pullPending[n] = <<>>
  /\ CanCompactThrough(hubLog, compactedThrough, boundary, snapshotThrough,
                        snapshotCoverage, tombstones, replicaWatermark, nodeActive,
                        Nodes, archivedEvents, localLog)

CompactPrefix(boundary) ==
  /\ CanCompactPrefix(boundary)
  /\ LET removed == PrefixEventsRemoved(hubLog, compactedThrough, boundary)
     IN /\ archivedEvents' = archivedEvents \union removed
        /\ archivedOffsets' =
             ArchiveOffsets(Events, hubLog, compactedThrough, boundary,
                            archivedOffsets)
        /\ hubLog' = CompactHubLog(hubLog, compactedThrough, boundary)
        /\ compactedThrough' = boundary
  /\ UNCHANGED <<hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen, pullBuf, pullPending,
                 snapshotThrough, snapshotCoverage, tombstones,
                 replicaWatermark, nodeActive, bootstrapped, bootstrapCoverage,
                 snapshotCursors>>

ColdResetNode(node) ==
  /\ localLog' = [localLog EXCEPT ![node] = {}]
  /\ cursors' = [cursors EXCEPT ![node] = [a \in Nodes |-> 0]]
  /\ pullBuf' = [pullBuf EXCEPT ![node] = <<>>]
  /\ pullPending' = [pullPending EXCEPT ![node] = <<>>]
  /\ bootstrapped' = [bootstrapped EXCEPT ![node] = FALSE]
  /\ bootstrapCoverage' = [bootstrapCoverage EXCEPT ![node] = {}]
  /\ nodeActive' = [nodeActive EXCEPT ![node] = FALSE]
  /\ UNCHANGED <<hubLog, hubAccepted, outQueue, pushStream, pushDurableLen,
                 compactedThrough, archivedEvents, archivedOffsets, snapshotThrough,
                 snapshotCoverage, tombstones, replicaWatermark, snapshotCursors>>

BootstrapFromSnapshot(node) ==
  /\ CanBootstrapFromSnapshot(snapshotThrough, bootstrapped, node)
  /\ LET cursorsForBootstrap ==
       CoverageCursors(snapshotCoverage, hubLog, compactedThrough,
                       archivedOffsets, archivedEvents, Author, Nodes)
     IN /\ localLog' = [localLog EXCEPT ![node] = snapshotCoverage]
        /\ cursors' = [cursors EXCEPT ![node] = cursorsForBootstrap]
        /\ pullBuf' = [pullBuf EXCEPT ![node] = <<>>]
        /\ pullPending' = [pullPending EXCEPT ![node] = <<>>]
        /\ bootstrapped' = [bootstrapped EXCEPT ![node] = TRUE]
        /\ bootstrapCoverage' = [bootstrapCoverage EXCEPT ![node] = snapshotCoverage]
        /\ nodeActive' = [nodeActive EXCEPT ![node] = TRUE]
  /\ UNCHANGED <<hubLog, hubAccepted, outQueue, pushStream, pushDurableLen,
                 compactedThrough, archivedEvents, archivedOffsets,
                 snapshotThrough, snapshotCoverage, tombstones,
                 replicaWatermark, snapshotCursors>>

Next ==
  \/ \E node \in Nodes, event \in Events : Enqueue(node, event)
  \/ \E node \in Nodes : StartPush(node)
  \/ \E node \in Nodes : PushCommitNext(node)
  \/ \E node \in Nodes : InterruptPush(node)
  \/ \E node \in Nodes : MalformedPush(node)
  \/ \E node \in Nodes : BeginPull(node)
  \/ \E node \in Nodes : PullSendNext(node)
  \/ \E node \in Nodes : InterruptPull(node)
  \/ \E node \in Nodes : MalformedPull(node)
  \/ \E node \in Nodes : Persist(node)
  \/ PublishSnapshot
  \/ \E node \in Nodes : ReportWatermark(node)
  \/ \E boundary \in 1..MaxHubLen : CompactPrefix(boundary)
  \/ \E node \in Nodes : ColdResetNode(node)
  \/ \E node \in Nodes : BootstrapFromSnapshot(node)

Spec ==
  Init /\ [][Next]_vars
    /\ \A node \in Nodes : WF_vars(BootstrapFromSnapshot(node))

Inv_IdempotentAppend ==
  \A i, j \in DOMAIN hubLog :
    (i # j) => (hubLog[i] # hubLog[j])

Inv_DurableOnlyPull ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      pullBuf[node][i] \in AllDurableEvents(hubLog)

Inv_PersistBeforeCursor ==
  \A node \in Nodes :
    nodeActive[node] =>
      PersistBeforeCursorOK(hubLog, localLog[node], cursors[node], Author,
                            compactedThrough, archivedEvents, archivedOffsets)

Inv_CursorWithinHub ==
  \A node \in Nodes, author \in Nodes :
    cursors[node][author] <= HubLenAbsolute(hubLog, compactedThrough)

Inv_AcceptedNotPullable ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      pullBuf[node][i] \notin hubAccepted

Inv_HubOffsetOrder ==
  \A node \in Nodes :
    \A i, j \in DOMAIN pullBuf[node] :
      (i < j) =>
        HubOffsetOfEvent(hubLog, pullBuf[node][i], compactedThrough)
          < HubOffsetOfEvent(hubLog, pullBuf[node][j], compactedThrough)

Inv_PaginationStable ==
  \A node \in Nodes :
    LET page == pullBuf[node] \o pullPending[node]
        minOff == MinUnseenOffset(hubLog, cursors[node], Author,
                                  compactedThrough)
    IN (page = <<>>)
       \/ ((Len(page) >= 1)
           /\ HubOffsetOfEvent(hubLog, page[1], compactedThrough) = minOff)

Inv_CursorMonotone ==
  \A node \in Nodes, author \in Nodes :
    \/ cursors[node][author] = 0
    \/ /\ cursors[node][author] <= HubLenAbsolute(hubLog, compactedThrough)
       /\ \/ cursors[node][author] <= compactedThrough
          \/ Author[EventAtOffset(hubLog, compactedThrough,
                                   cursors[node][author])] = author

Inv_PartialPush ==
  \A node \in Nodes :
    LET durable == AllDurableEvents(hubLog) \union archivedEvents
    IN /\ \A i \in 1..pushDurableLen[node] :
         pushStream[node][i] \in durable
       /\ \A i \in DOMAIN pushStream[node] :
         (i > pushDurableLen[node]) => pushStream[node][i] \notin durable

Inv_PartialPull ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      LET event == pullBuf[node][i]
          offset == HubOffsetOfEvent(hubLog, event, compactedThrough)
          author == Author[event]
      IN cursors[node][author] < offset

Inv_MalformedLine ==
  /\ Inv_PartialPush
  /\ Inv_PartialPull

Inv_NoSilentLoss ==
  \A node \in Nodes :
    (nodeActive[node] /\ replicaWatermark[node] >= compactedThrough)
      => archivedEvents \subseteq localLog[node]

Inv_CompactionSafe ==
  /\ compactedThrough <= snapshotThrough
  /\ archivedEvents \subseteq snapshotCoverage

Inv_TombstoneRetained ==
  tombstones \subseteq snapshotCoverage \union AllDurableEvents(hubLog)

Inv_BootstrapCoverage ==
  \A node \in Nodes :
    bootstrapped[node] => bootstrapCoverage[node] \subseteq localLog[node]

Live_InactiveBootstrap ==
  \A node \in Nodes :
    (snapshotThrough > 0)
      ~> (bootstrapped[node]
          /\ bootstrapCoverage[node] \subseteq localLog[node])

====
