===description===
Writing to a property (`$w->value = ...`) declared on an `@extends`-fixed
ANCESTOR (not the receiver class itself) checks against the ancestor's
fixed binding, not the receiver's own, same-lettered template — the
write-side counterpart of the already-fixed read-side property access.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    public $value;
}

/**
 * @template T
 * @extends Box<int>
 */
class Wrapper extends Box {
    /** @var T */
    public $extra;
}

/** @param Wrapper<string> $w */
function collision(Wrapper $w): void {
    $w->value = "not an int";
}

// Cross-directional check: Wrapper's OWN member (`extra`, declared directly
// on Wrapper) must still accept its own T (string), not the ancestor's int.
/** @param Wrapper<string> $w */
function ownMemberStillCorrect(Wrapper $w): void {
    $w->extra = "a string is fine here";
}
===expect===
InvalidPropertyAssignment@21:4-21:28: Property $value expects 'int', cannot assign '"not an int"'
