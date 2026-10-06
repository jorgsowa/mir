===description===
Positive counterpart: the true branch of `is_iterable()` is unaffected by the
false-branch fix and still narrows `Box|array` down to iterable atoms only,
so `foreach` over the array member raises nothing.
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {}

/** @param Box|array<int,int> $x */
function f($x): void {
    if (is_iterable($x)) {
        foreach ($x as $v) {}
    }
}
