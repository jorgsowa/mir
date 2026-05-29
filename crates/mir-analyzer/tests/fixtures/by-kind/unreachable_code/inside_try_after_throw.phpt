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
MissingThrowsDocblock@4:9-4:37: Exception Exception is thrown but not declared in @throws
UnreachableCode@5:9-5:28: Unreachable code detected
