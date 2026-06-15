===description===
after throw expression assignment
===file===
<?php
function test(): void {
    $value = throw new RuntimeException('stop');
    echo 'unreachable';
}
===expect===
UnreachableCode@4:4-4:23: Unreachable code detected
