===description===
do while control var not reported
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(): void {
    do {
        $run = false;
        if (time() % 3 === 0) {
            continue;
        }
        $run = true;
    } while ($run);
}
===expect===
