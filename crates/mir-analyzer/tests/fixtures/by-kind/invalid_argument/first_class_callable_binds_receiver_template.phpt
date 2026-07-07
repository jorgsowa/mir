===description===
FN: `$box->set(...)` (first-class callable syntax) built its closure type
from the method's raw, unsubstituted params — unlike the direct-call path,
it never substituted the receiver's own bound type params (`Box<int>`'s
T -> int), so calling the resulting closure with a mismatched argument was
silently accepted where the direct call correctly rejects it.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $value */
    public function __construct($value) {}

    /** @param T $x */
    public function set($x): void {}
}

$box = new Box(1);
$fn = $box->set(...);
$fn("bad-not-int");
===expect===
InvalidArgument@13:4-13:17: Argument $x of {closure}() expects 'int', got '"bad-not-int"'
