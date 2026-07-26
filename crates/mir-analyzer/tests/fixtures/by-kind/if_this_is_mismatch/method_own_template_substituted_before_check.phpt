===description===
An @if-this-is constraint referencing the METHOD's own @template (not the
class's) was never substituted before the subtype comparison — the check
ran BEFORE this call's own inferred template bindings existed, so the
receiver-vs-constraint comparison always saw a bare, unsubstituted
template atom and could never actually contradict. Moved the check to
run after the method's own bindings are inferred from the call's
arguments.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @param T $value */
    public function __construct($value) {}

    /**
     * @template U
     * @if-this-is Box<U>
     * @param U $val
     */
    public function replace($val): void {}
}

$box = new Box('hi');
$box->replace(42);
===expect===
IfThisIsMismatch@18:0-18:17: Cannot call Box::replace() — @if-this-is requires $this to be 'Box<U>', but it is 'Box<string>'
