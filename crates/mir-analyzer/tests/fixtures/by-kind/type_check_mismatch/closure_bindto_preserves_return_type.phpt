===description===
Closure->bindTo preserves return type from original closure
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}

$closure = function(): int { return 42; };
/** @mir-check $closure is Closure(): positive-int */

$bound = $closure->bindTo(new B());
// bindTo() should return Closure(): int|null, not mixed
/** @mir-check $bound is Closure(): positive-int|null */

if ($bound !== null) {
    $result = $bound();
    /** @mir-check $result is int */
}
===expect===
