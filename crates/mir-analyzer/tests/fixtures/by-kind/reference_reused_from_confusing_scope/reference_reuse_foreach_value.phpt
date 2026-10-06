===description===
Reference reuse foreach value
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<int> */
$arr = [];

foreach ($arr as &$var) {
    $var += 1;
}

$var = "foo";
