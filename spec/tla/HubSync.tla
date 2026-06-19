---- MODULE HubSync ----
(***************************************************************************
  Track hub sync protocol — root specification (ADR 0004 / ADR 0006).

  Phase 2 models:
    - idempotent push with streaming commit and mid-stream abort
    - per-authoring-node cursors and incremental pull delivery
    - pull interrupt without cursor advance for undelivered tail

  Abstractions (documented in spec/tla/README.md):
    - no compaction or snapshots (Phase 4)
 ***************************************************************************)
EXTENDS Common, Hub, Node, Network

CONSTANTS Nodes, Events, MaxHubLen, PageLimit, MaxPushStream

\* Authorship for the finite CI model (node 1 authors events 1 and 2).
Author ==
  [event \in Events |-> IF event \in {1, 2} THEN 1 ELSE 2]

ZeroCursors ==
  [n \in Nodes |-> [a \in Nodes |-> 0]]

VARIABLES hubLog, hubAccepted, localLog, cursors, outQueue,
          pushStream, pushDurableLen, pullBuf, pullPending

vars ==
  <<hubLog, hubAccepted, localLog, cursors, outQueue,
    pushStream, pushDurableLen, pullBuf, pullPending>>

TypeOK ==
  /\ hubLog \in Seq(Events)
  /\ hubAccepted \subseteq Events
  /\ Len(hubLog) <= MaxHubLen
  /\ localLog \in [Nodes -> SUBSET Events]
  /\ cursors \in [Nodes -> [Nodes -> Nat]]
  /\ \A n \in Nodes, a \in Nodes : cursors[n][a] <= HubLen(hubLog)
  /\ outQueue \in [Nodes -> Seq(Events)]
  /\ pushStream \in [Nodes -> Seq(Events)]
  /\ pushDurableLen \in [Nodes -> Nat]
  /\ pullBuf \in [Nodes -> Seq(Events)]
  /\ pullPending \in [Nodes -> Seq(Events)]
  /\ \A n \in Nodes :
       pushDurableLen[n] <= Len(pushStream[n])
  /\ \A n \in Nodes :
       \A i \in DOMAIN pullBuf[n] :
         \E j \in DOMAIN hubLog : hubLog[j] = pullBuf[n][i]

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

CanEnqueue(node, event) ==
  /\ Author[event] = node
  /\ event \notin AllDurableEvents(hubLog)
  /\ event \notin {outQueue[node][i] : i \in DOMAIN outQueue[node]}
  /\ event \notin {pushStream[node][i] : i \in DOMAIN pushStream[node]}

Enqueue(node, event) ==
  /\ CanEnqueue(node, event)
  /\ outQueue' = [outQueue EXCEPT ![node] = Append(@, event)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors,
                 pushStream, pushDurableLen, pullBuf, pullPending>>

CanStartPush(node) ==
  /\ StreamIdle(pushStream[node], pushDurableLen[node])
  /\ outQueue[node] # <<>>
  /\ Len(hubLog) + Cardinality(hubAccepted) < MaxHubLen

StartPush(node) ==
  /\ CanStartPush(node)
  /\ LET batch == TakeFirstEvents(outQueue[node], MaxPushStream)
         taken == Len(batch)
     IN /\ pushStream' = [pushStream EXCEPT ![node] = batch]
        /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = 0]
        /\ outQueue' = [outQueue EXCEPT ![node] =
             RemainderAfterTake(@, taken)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, pullBuf, pullPending>>

CanPushCommitNext(node) ==
  /\ pushStream[node] # <<>>
  /\ pushDurableLen[node] < Len(pushStream[node])
  /\ Len(hubLog) + Cardinality(hubAccepted) < MaxHubLen

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
  /\ UNCHANGED <<localLog, cursors, outQueue, pullBuf, pullPending>>

CanAbortPush(node) ==
  /\ pushStream[node] # <<>>
  /\ pushDurableLen[node] < Len(pushStream[node])

AbortPushStream(node) ==
  /\ CanAbortPush(node)
  /\ LET remainder == StreamRemainder(pushStream[node], pushDurableLen[node])
     IN /\ outQueue' = [outQueue EXCEPT ![node] = PrependSeq(remainder, @)]
        /\ pushStream' = [pushStream EXCEPT ![node] = <<>>]
        /\ pushDurableLen' = [pushDurableLen EXCEPT ![node] = 0]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, pullBuf, pullPending>>

MalformedPush(node) == AbortPushStream(node)
InterruptPush(node) == AbortPushStream(node)

CanBeginPull(node) ==
  /\ pullBuf[node] = <<>>
  /\ pullPending[node] = <<>>
  /\ PullWindow(hubLog, cursors[node], PageLimit, Author) # <<>>

BeginPull(node) ==
  /\ CanBeginPull(node)
  /\ pullPending' = [pullPending EXCEPT ![node] =
       PullWindow(hubLog, cursors[node], PageLimit, Author)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen, pullBuf>>

CanPullSendNext(node) == pullPending[node] # <<>>

PullSendNext(node) ==
  /\ CanPullSendNext(node)
  /\ LET event == Head(pullPending[node])
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Append(@, event)]
        /\ pullPending' = [pullPending EXCEPT ![node] = Tail(@)]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen>>

CanAbortPull(node) ==
  pullPending[node] # <<>> \/ pullBuf[node] # <<>>

AbortPullStream(node) ==
  /\ CanAbortPull(node)
  /\ pullPending' = [pullPending EXCEPT ![node] = <<>>]
  /\ pullBuf' = [pullBuf EXCEPT ![node] = <<>>]
  /\ UNCHANGED <<hubLog, hubAccepted, localLog, cursors, outQueue,
                 pushStream, pushDurableLen>>

MalformedPull(node) == AbortPullStream(node)
InterruptPull(node) == AbortPullStream(node)

CanPersist(node) == pullBuf[node] # <<>>

Persist(node) ==
  /\ CanPersist(node)
  /\ LET event == Head(pullBuf[node])
         offset == HubOffsetOfEvent(hubLog, event)
         author == Author[event]
     IN /\ pullBuf' = [pullBuf EXCEPT ![node] = Tail(@)]
        /\ localLog' = [localLog EXCEPT ![node] = @ \union {event}]
        /\ cursors' = [cursors EXCEPT ![node][author] = offset]
        /\ UNCHANGED <<hubLog, hubAccepted, outQueue,
                       pushStream, pushDurableLen, pullPending>>

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
    LET page == pullBuf[node] \o pullPending[node]
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

Inv_PartialPush ==
  \A node \in Nodes :
    /\ \A i \in 1..pushDurableLen[node] :
         pushStream[node][i] \in AllDurableEvents(hubLog)
    /\ \A i \in DOMAIN pushStream[node] :
         (i > pushDurableLen[node])
           => pushStream[node][i] \notin AllDurableEvents(hubLog)

Inv_PartialPull ==
  \A node \in Nodes :
    \A i \in DOMAIN pullBuf[node] :
      LET event == pullBuf[node][i]
          offset == HubOffsetOfEvent(hubLog, event)
          author == Author[event]
      IN cursors[node][author] < offset

Inv_MalformedLine ==
  /\ Inv_PartialPush
  /\ Inv_PartialPull

====
