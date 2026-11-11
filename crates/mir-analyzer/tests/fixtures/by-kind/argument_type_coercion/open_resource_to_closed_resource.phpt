===description===
A handle not known to be closed is a coercion to `closed-resource`; after fclose() the variable is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param closed-resource $handle */
function archive($handle): void {}

function run(string $path): void {
    $handle = fopen($path, 'r');
    if ($handle === false) {
        return;
    }
    archive($handle);
//          ^^^^^^^ ArgumentTypeCoercion: Argument $handle of archive() expects 'closed-resource', got 'resource' — coercion may fail at runtime
    fclose($handle);
    archive($handle);
}
