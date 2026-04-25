===file===
<?php
function test(): void {
    $value = throw new RuntimeException('stop');
    echo 'unreachable';
}
===expect===
UnreachableCode: Unreachable code detected
