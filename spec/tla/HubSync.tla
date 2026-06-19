---- MODULE HubSync ----
(***************************************************************************
  Track hub sync protocol — root specification (ADR 0004 / ADR 0006).

  Phase 1 models:
    - idempotent push (accepted, then durable promotion)
    - per-authoring-node cursors and paged pull
    - local persist before cursor advance

  Abstractions (documented in spec/tla/README.md):
    - atomic push/pull steps (no Network.tla interleaving yet)
    - no compaction or snapshots
 ***************************************************************************)
EXTENDS Common, Hub, Node

CONSTANTS Nodes, Events, MaxHubLen, PageLimit

\* Authorship for the finite CI model (node 1 authors events 1 and 2).
Author ==
  [event \in Events |-> IF event \in {1, 2} THEN 1 ELSE 2]

ZeroCursors ==
  [n \in Nodes |-> [a \in Nodes |-> 0]]

VARIABLES hubLog, hubAccepted, localLog, cursors, outQueue, pullBuf

vars == <<hubLog, hubAccepted, localLog, cursors, outQueue, pullBuf>>

TypeOK ==
  /\ hubLog \in Seq(Events)
  /\ hubAccepted \subseteq Events
  /\ Len(hubLog) <= MaxHubLen
  /\ localLog \in [Nodes -> SUBSET Events]
  /\ cursors \in [Nodes -> [Nodes -> Nat]]
  /\ \A n \in Nodes, a \in Nodes : cursors[n][a] <= HubLen(hubLog)
  /\ outQueue \in [Nodes -> Seq(Events)]
  /\ pullBuf \in [Nodes -> Seq(Events)]
  /\ \A n \in Nodes :
       \A i \in DOMAIN pullBuf[n] :
         \E j \in DOMAIN hubLog : hubLog[j] = pullBuf[n][i]

Init ==
  /\ hubLog = <<>>
  /\ hubAccepted = {}
  /\ localLog = [n \in Nodes |-> {}]
  /\ cursors = ZeroCursors
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
  /\ pullBuf[node] = <<>>
  /\ PullWindow(hubLog, cursors[node], PageLimit, Author) # <<>>

PullDeliver(node) ==
  /\ CanPullDeliver(node)
  /\ pullBuf' = [pullBuf EXCEPT ![node] =
       PullWindow(hubLog, cursors[node], PageLimit, Author)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue>>

CanPersist(node) == pullBuf[node] # <<>>

Persist(node) ==
  /\ CanPersist(node)
  /\ LET event == Head(pullBuf[node])
         offset == HubOffsetOfEvent(hubLog, event)
         author == Author[event]
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Tail(@)]
        /\ localLog' = [localLog EXCEPT ![node] = @ \union {event}]
        /\ cursors' = [cursors EXCEPT ![node][author] = offset]
        /\ UNCHANGED <<hubLog, hubAccepted, outQueue>>

Next ==
  \/ \E node \in Nodes, event \in Events : Enqueue(node, event)
  \/ \E node \in Nodes : Push(node)
  \/ Promote
  \/ \E node \in Nodes : PullDeliver(node)
  \/ \E node \in Nodes : Persist(node)

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
    PersistBeforeCursorOK(hubLog, localLog[node], cursors[node], Author)

Inv_CursorWithinHub ==
  \A node \in Nodes, author \in Nodes :
    cursors[node][author] <= HubLen(hubLog)

Inv_AcceptedNotPullable ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      pullBuf[node][i] \notin hubAccepted

Inv_HubOffsetOrder ==
  \A node \in Nodes :
    \A i, j \in DOMAIN pullBuf[node] :
      (i < j) =>
        HubOffsetOfEvent(hubLog, pullBuf[node][i])
          < HubOffsetOfEvent(hubLog, pullBuf[node][j])

Inv_PaginationStable ==
  \A node \in Nodes :
    LET page == pullBuf[node]
        minOff == MinUnseenOffset(hubLog, cursors[node], Author)
    IN (page = <<>>)
       \/ ((Len(page) >= 1)
           /\ HubOffsetOfEvent(hubLog, page[1]) = minOff)

Inv_CursorMonotone ==
  \A node \in Nodes, author \in Nodes :
    \/ cursors[node][author] = 0
    \/ \E i \in DOMAIN hubLog :
         /\ Author[hubLog[i]] = author
         /\ cursors[node][author] = i

====
