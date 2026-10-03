===description===
does not report too many with spread
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_one(int $a): void {}
$arr = [1, 2, 3];
takes_one(...$arr);
===expect===
