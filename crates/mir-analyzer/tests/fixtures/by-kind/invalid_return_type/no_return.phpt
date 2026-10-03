===description===
No return
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$bar = ["foo", "bar"];

$bam = array_map(
    function(string $a): string {
    },
    $bar
);
===expect===
