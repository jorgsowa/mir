===description===
A bare class reference may hold an instance whose `null`-defaulted template is filled, so the default doesn't apply
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface P {}

/** @template TParent of P|null = null */
class Node implements P {
    /** @return TParent */
    public function end(): ?P { return null; }
}

/** @param Node $n */
function f(P $n): void {
    $p = $n->end();
    /** @mir-check $p is P|null */
    echo 1;
}
