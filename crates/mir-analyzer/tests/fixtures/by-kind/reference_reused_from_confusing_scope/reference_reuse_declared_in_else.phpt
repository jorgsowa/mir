===description===
Reference reuse declared in else
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<int> */
$arr = [];

if (!isset($arr[0])) {
} else {
    $var = &$arr[0];
    $var += 1;
}

$var = "foo";
