===description===
after throw expression assignment
===file===
<?php
function test(): void {
    $value = throw new RuntimeException('stop');
    echo 'unreachable';
//  ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
