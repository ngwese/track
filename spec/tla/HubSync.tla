---- MODULE HubSync ----
(***************************************************************************
  Track hub sync protocol — root specification (ADR 0004 / ADR 0006).

  Phase 0 (v0) models:
    - idempotent push (accepted, then durable promotion)
    - cursor-based pull with paging
  - local persist before cursor advance

  Abstractions (documented in spec/tla/README.md):
    - one workspace cursor per syncing node (not per authoring node yet)
    - atomic push/pull steps (no Network.tla interleaving yet)
    - no compaction or snapshots
 ***************************************************************************)
EXTENDS Common, Hub, Node

CONSTANTS Nodes, Events, MaxHubLen, PageLimit

\* Authorship for the finite CI model (node 1 authors events 1 and 2).
Author ==
  [event \in Events |-> IF event \in {1, 2} THEN 1 ELSE 2]


VARIABLES hubLog, hubAccepted, localLog, cursors, outQueue, pullBuf

vars == <<hubLog, hubAccepted, localLog, cursors, outQueue, pullBuf>>

TypeOK ==
  /\ hubLog \in Seq(Events)
  /\ hubAccepted \subseteq Events
  /\ Len(hubLog) <= MaxHubLen
  /\ localLog \in [Nodes -> SUBSET Events]
  /\ cursors \in [Nodes -> Nat]
  /\ \A n \in Nodes : cursors[n] <= HubLen(hubLog)
  /\ outQueue \in [Nodes -> Seq(Events)]
  /\ pullBuf \in [Nodes -> Seq(Events)]
  /\ \A n \in Nodes :
       \A i \in DOMAIN pullBuf[n] :
         \E j \in DOMAIN hubLog : hubLog[j] = pullBuf[n][i]

Init ==
  /\ hubLog = <<>>
  /\ hubAccepted = {}
  /\ localLog = [n \in Nodes |-> {}]
  /\ cursors = [n \in Nodes |-> 0]
  /\ outQueue = [n \in Nodes |-> <<>>]
  /\ pullBuf = [n \in Nodes |-> <<>>]

CanEnqueue(node, event) ==
  /\ Author[event] = node
  /\ event \notin AllDurableEvents(hubLog)
  /\ event \notin {outQueue[node][i] : i \in DOMAIN outQueue[node]}

Enqueue(node, event) ==
  /\ CanEnqueue(node, event)
  /\ outQueue' = [outQueue EXCEPT ![node] = Append(@, event)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, pullBuf>>

CanPush(node) ==
  /\ outQueue[node] # <<>>
  /\ Len(hubLog) + Cardinality(hubAccepted) < MaxHubLen

Push(node) ==
  /\ CanPush(node)
  /\ LET event == Head(outQueue[node])
         accepted == PushAccept(hubLog, hubAccepted, event)
     IN /\ outQueue' = [outQueue EXCEPT ![node] = Tail(@)]
        /\ hubAccepted' = accepted.hubAcceptedNew
        /\ UNCHANGED <<hubLog, localLog, cursors, pullBuf>>

Promote ==
  /\ hubAccepted # {}
  /\ LET promoted == PromoteAccepted(hubLog, hubAccepted)
     IN /\ promoted.promoted
        /\ hubLog' = promoted.hubLogNew
        /\ hubAccepted' = promoted.hubAcceptedNew
        /\ UNCHANGED <<localLog, cursors, outQueue, pullBuf>>

CanPullDeliver(node) ==
  /\ cursors[node] < HubLen(hubLog)
  /\ pullBuf[node] = <<>>

PullDeliver(node) ==
  /\ CanPullDeliver(node)
  /\ pullBuf' = [pullBuf EXCEPT ![node] =
       PullWindow(hubLog, cursors[node], PageLimit)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue>>

CanPersist(node) == pullBuf[node] # <<>>

Persist(node) ==
  /\ CanPersist(node)
  /\ LET event == Head(pullBuf[node])
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Tail(@)]
        /\ localLog' = [localLog EXCEPT ![node] = @ \union {event}]
        /\ UNCHANGED <<hubLog, hubAccepted, cursors, outQueue>>

CanAdvanceCursor(node) ==
  /\ cursors[node] < HubLen(hubLog)
  /\ hubLog[cursors[node] + 1] \in localLog[node]

AdvanceCursor(node) ==
  /\ CanAdvanceCursor(node)
  /\ cursors' = [cursors EXCEPT ![node] = @ + 1]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, outQueue, pullBuf>>

Next ==
  \/ \E node \in Nodes, event \in Events : Enqueue(node, event)
  \/ \E node \in Nodes : Push(node)
  \/ Promote
  \/ \E node \in Nodes : PullDeliver(node)
  \/ \E node \in Nodes : Persist(node)
  \/ \E node \in Nodes : AdvanceCursor(node)

Spec == Init /\ [][Next]_vars

Inv_IdempotentAppend ==
  \A i, j \in DOMAIN hubLog :
    (i # j) => (hubLog[i] # hubLog[j])

Inv_DurableOnlyPull ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      \E j \in DOMAIN hubLog : hubLog[j] = pullBuf[node][i]

Inv_PersistBeforeCursor ==
  \A node \in Nodes :
    PersistBeforeCursorOK(hubLog, localLog[node], cursors[node])

Inv_CursorWithinHub ==
  \A node \in Nodes : cursors[node] <= HubLen(hubLog)

Inv_AcceptedNotPullable ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      pullBuf[node][i] \notin hubAccepted

====
