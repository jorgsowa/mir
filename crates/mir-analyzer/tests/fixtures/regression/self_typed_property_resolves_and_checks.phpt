===description===
A bare `self`/`static`/`parent` in a property's own declared type (e.g.
`@var self|null`) resolves to the receiver's concrete class instead of
staying an unbound `self()` (read side); writing through a self/static/
parent-typed receiver now runs the same property-type check a plain
class-typed receiver already gets (write side).
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Node {
    /** @var self|null */
    public $next;
    /** @var T */
    public $value;
}

/** @param Node<int> $n */
function readsNext(Node $n): void {
    $x = $n->next;
    /** @mir-check $x is Node<int>|null */
    $_ = 1;
}

/** @param Node<int> $n */
function writeThroughSelfTypedReceiver(Node $n): void {
    $x = $n->next;
    if ($x !== null) {
        $x->value = "wrong";
    }
}
===expect===
InvalidPropertyAssignment@21:8-21:27: Property $value expects 'int', cannot assign '"wrong"'
