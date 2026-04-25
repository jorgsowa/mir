===file===
<?php
function test(): void {
    try {
        throw new Exception('stop');
        echo 'unreachable';
    } catch (Exception) {
    }
}
===expect===
UnreachableCode: Unreachable code detected
