===description===
Reference assignment to array offset: no issue, type is traceable
===config===
<mir>
  <issueHandlers>
    <Trace errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<string, string> */
$arr = [];

/** @var non-empty-list<string> */
$foo = ["foo"];

$bar = &$arr[$foo[0]];

/** @trace $bar */;
