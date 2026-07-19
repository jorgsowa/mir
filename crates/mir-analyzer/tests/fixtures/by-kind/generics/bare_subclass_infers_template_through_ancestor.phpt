===description===
Inferring a function's own `@template X` from a `Collection<X>`-shaped
param binds X correctly when the arg is a bare subclass that doesn't
redeclare @template (`class IntBox extends Box {}`), the same way a
directly-generic arg class already does.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingThrowsDocblock
===file===
<?php
/** @template T */
interface Collection {}

/**
 * @template T
 * @implements Collection<T>
 */
class Box implements Collection {}

class IntBox extends Box {}

/**
 * @template X
 * @param Collection<X> $c
 * @return X
 */
function firstOf(Collection $c) {
    throw new \Exception();
}

function test(): void {
    /** @var IntBox<int> $box */
    $box = new IntBox();
    $x = firstOf($box);
    /** @mir-check $x is int */
    $_ = $x;
}
===expect===
