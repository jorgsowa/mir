===description===
Method call on int, string and array receivers; the result is mixed.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(int $n, string $s, array $a): void {
    $r = $n->go();
//       ^^^^^^^^ InvalidMethodCall: Cannot call method go() on non-object type 'int'
    /** @mir-check $r is mixed */
    echo $s->go();
//       ^^^^^^^^ InvalidMethodCall: Cannot call method go() on non-object type 'string'
    echo $a->go();
//       ^^^^^^^^ InvalidMethodCall: Cannot call method go() on non-object type 'array'
}
