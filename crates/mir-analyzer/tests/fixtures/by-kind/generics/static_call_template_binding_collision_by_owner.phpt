===description===
A static call (`$w::make()`) resolving a class-level `@return T`
declared on an `@extends`-fixed ANCESTOR (not the receiver class itself)
must use the ancestor's fixed binding, not the receiver's own,
same-lettered template — the two are unrelated template scopes despite
sharing the conventional name `T`.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @return T */
    public static function make(): mixed {
        return null;
    }
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
    /** @mir-check $w::make() is int */
    $_ = $w::make();
}

// Cross-directional check: Wrapper's OWN member (`extra`, declared directly
// on Wrapper) must still resolve using Wrapper's own T.
/** @param Wrapper<string> $w */
function ownMemberStillCorrect(Wrapper $w): void {
    /** @mir-check $w->extra is string */
    $_ = $w->extra;
}
===expect===
