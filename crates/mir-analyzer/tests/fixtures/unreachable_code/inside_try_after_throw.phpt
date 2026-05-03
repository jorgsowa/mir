===description===
inside try after throw
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
UnreachableCode@5:8: Unreachable code detected
===ignore===
TODO
