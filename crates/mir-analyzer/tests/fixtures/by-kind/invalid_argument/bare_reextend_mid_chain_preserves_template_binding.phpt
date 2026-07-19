===description===
A middle hop in a 3+-level @extends chain that bare-re-extends its parent
(no own @template, no <T> on extends) must not drop the ancestor template
binding — resolve_through zipped an edge's OWN-declarations-only template
list against the subclass's @extends args, instead of the edge's effective
(ancestor-walked) list. Since the bare hop declares no @template of its own,
this produced an empty bindings map, leaving the inherited ancestor template
unsubstituted and silently masking a real argument type mismatch.
===config===
suppress=ForbiddenCode
===file===
<?php
/** @template TBox */
class Box {}

/**
 * @template TContainer
 * @extends Box<TContainer>
 */
class Container extends Box {}

class NamedContainer extends Container {}

/**
 * @template TTyped
 * @extends NamedContainer<TTyped>
 */
class TypedContainer extends NamedContainer {}

/** @param Box<string> $b */
function takesBoxOfString(Box $b): void { var_dump($b); }

/** @param TypedContainer<int> $c */
function mismatchIsCaught(TypedContainer $c): void {
    takesBoxOfString($c);
}

/** @param TypedContainer<string> $c */
function matchIsAccepted(TypedContainer $c): void {
    takesBoxOfString($c);
}
===expect===
InvalidArgument@24:21-24:23: Argument $b of takesBoxOfString() expects 'Box<string>', got 'TypedContainer<int>'
