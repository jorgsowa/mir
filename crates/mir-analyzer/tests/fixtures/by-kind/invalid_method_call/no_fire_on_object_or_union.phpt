===description===
No InvalidMethodCall for objects, mixed, or scalar|object unions.
===config===
<mir>
  <issueHandlers>
    <MixedMethodCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public function go(): int { return 1; }
}

/** @param Box|false $u */
function f(Box $b, mixed $m, $u, ?Box $n): void {
    echo $b->go();
    echo $m->go();
    echo $u->go();
    echo $n?->go();
}
