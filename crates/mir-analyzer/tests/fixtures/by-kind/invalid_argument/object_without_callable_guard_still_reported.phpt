===description===
Negative control for the L30 fix: without an `is_callable()` guard, passing
a bare `object` where `callable` is expected must still be flagged —
narrowing must not leak into the un-guarded case.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param callable $fn */
function needsCallable(callable $fn): void {}

/** @param object $x */
function test(object $x): void {
    needsCallable($x);
//                ^^ InvalidArgument: Argument $fn of needsCallable() expects 'callable', got 'object'
}
===expect===
