---- MODULE Hub ----
(***************************************************************************
  Hub-side transitions: accept push, promote accepted events to durable log.
 ***************************************************************************)
EXTENDS Common

PushAccept(hubLog, hubAccepted, event) ==
  IF event \in AllKnownHubEvents(hubLog, hubAccepted)
  THEN [hubAcceptedNew |-> hubAccepted, duplicate |-> TRUE]
  ELSE [hubAcceptedNew |-> hubAccepted \union {event}, duplicate |-> FALSE]

PromoteAccepted(hubLog, hubAccepted) ==
  IF hubAccepted = {}
  THEN [hubLogNew |-> hubLog,
        hubAcceptedNew |-> hubAccepted,
        promoted |-> FALSE]
  ELSE LET event == CHOOSE e \in hubAccepted : TRUE
       IN [hubLogNew |-> Append(hubLog, event),
           hubAcceptedNew |-> hubAccepted \ {event},
           promoted |-> TRUE,
           event |-> event]

====
