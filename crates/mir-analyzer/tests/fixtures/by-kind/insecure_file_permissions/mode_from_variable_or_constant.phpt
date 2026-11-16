===description===
A mode held in a variable or constant with one literal value is checked like one
written at the call; a parameter isn't.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const SHARED_MODE = 0666;

function publish(string $file, int $mode): void {
    chmod($file, SHARED_MODE);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureFilePermissions: Insecure file permissions: chmod() mode 0666 lets every user on the system write the file
    $open = 0777;
    chmod($file, $open);
//  ^^^^^^^^^^^^^^^^^^^ InsecureFilePermissions: Insecure file permissions: chmod() mode 0777 lets every user on the system write the file
    chmod($file, $mode);
    $private = 0600;
    chmod($file, $private);
}
