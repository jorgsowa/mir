===description===
A nullable string includes null — comparison against null should not fire.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(?string $s): void {
    if ($s === null) {}
    if ($s === "hello") {}
}
===expect===
