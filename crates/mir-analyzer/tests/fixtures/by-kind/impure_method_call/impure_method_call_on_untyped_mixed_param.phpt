===description===
Calling a method on an untyped (mixed) parameter inside a @pure function is
still flagged — the check doesn't need the receiver's resolved type at all,
so it must not be skipped just because the parameter has no type hint.
===config===
suppress=MissingParamType,MixedMethodCall
===file===
<?php
namespace Baz;

/** @pure */
function run($a): void {
    $a->mutate();
}
===expect===
ImpureMethodCall@6:4-6:16: Calling impure method mutate() in a pure or immutable context
