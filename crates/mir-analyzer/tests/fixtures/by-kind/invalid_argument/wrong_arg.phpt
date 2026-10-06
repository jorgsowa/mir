===description===
Wrong arg
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
    function(int $a): int {
        return $a + 1;
    },
    $bar
);
