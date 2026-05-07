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
MissingThrowsDocblock@4:8: Exception Exception is thrown but not declared in @throws
UnreachableCode@5:8: Unreachable code detected
