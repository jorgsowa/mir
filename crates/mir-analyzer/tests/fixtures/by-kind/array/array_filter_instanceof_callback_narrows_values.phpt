===description===
An `array_filter()` callback testing `$x instanceof C` narrows the kept values to `C`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {}
final class A implements I {}
/** @param list<I> $xs */
function f(array $xs): void {
    $as = array_filter($xs, fn (I $x): bool => $x instanceof A);
    /** @mir-check $as is array<int, A> */
    echo 1;
    $bs = array_filter($xs, static function ($x) { return $x instanceof A; });
    /** @mir-check $bs is array<int, A> */
    echo 1;
}
