===description===
is_resource() is false for a closed handle, so testing a variable right after fclose() is redundant; a reopened one is not.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(string $path): void {
    $handle = fopen($path, 'r');
    if ($handle === false) {
        return;
    }
    fclose($handle);
    if (is_resource($handle)) {
//      ^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo 'still open';
    }
    $handle = fopen($path, 'r');
    if (is_resource($handle)) {
        echo 'open again';
    }
}
