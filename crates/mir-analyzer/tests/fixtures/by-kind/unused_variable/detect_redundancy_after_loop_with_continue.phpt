===description===
Detect redundancy after loop with continue
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$gap = null;
//<^^^^ UnusedVariable: Variable $gap is never read

foreach ([1, 2, 3] as $_) {
    if (rand(0, 1)) {
        continue;
    }

    $gap = "asa";
    throw new Exception($gap);
}
