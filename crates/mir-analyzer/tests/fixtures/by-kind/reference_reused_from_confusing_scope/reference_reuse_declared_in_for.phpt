===description===
Reference reuse declared in for
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var list<int> */
$arr = [];

for ($i = 0; $i < count($arr); ++$i) {
    $var = &$arr[$i];
    $var += 1;
}

$var = "foo";

===expect===
