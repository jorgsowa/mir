===description===
FP: an unresolved template nested inside an intersection param type (`T&Iface`,
no concrete `@extends` binding) wasn't recognized as template-containing, so
overriding it with a different intersection member ran the strict structural
param-narrowing check instead of being skipped like a bare `@param T` is.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Countable2 {}
interface ArrayAccessLike {}
interface IteratorLike {}

/**
 * @template T of Countable2
 */
abstract class Base {
    /** @param T&ArrayAccessLike $x */
    abstract public function accept($x): void;
}

class Impl extends Base {
    /** @param T&IteratorLike $x */
    public function accept($x): void {}
}
===expect===
