===description===
inside try after throw
===file===
<?php
function test(): void {
    try {
        throw new Exception('stop');
        echo 'unreachable';
//      ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    } catch (Exception) {
    }
}
===expect===
