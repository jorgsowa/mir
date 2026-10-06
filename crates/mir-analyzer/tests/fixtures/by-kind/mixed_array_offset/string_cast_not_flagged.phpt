===description===
MixedArrayOffset does NOT fire after an explicit (string) cast — the cast produces a concrete string type
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var mixed $x */
$x = 'hello';
$arr = ['hello' => 1, 'world' => 2];
$val = $arr[(string) $x];
