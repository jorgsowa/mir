===description===
Two specific inferred integer literals that differ can never be ===.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $a = 5;
    $b = 6;
    if ($a === $b) {}
//      ^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '5' and '6' is always false — these types can never be identical
}
