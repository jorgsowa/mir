===description===
Identical literals can be ===; no diagnostic should fire.
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
    $b = 5;
    if ($a === $b) {}
    $s1 = "foo";
    $s2 = "foo";
    if ($s1 === $s2) {}
}
