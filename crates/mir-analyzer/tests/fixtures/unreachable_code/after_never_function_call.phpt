===description===
after never function call
===file===
<?php
function stop(): never {
    throw new RuntimeException('stop');
}

function test(): void {
    stop();
    echo 'unreachable';
}
===expect===
MissingThrowsDocblock@3:4: Exception RuntimeException is thrown but not declared in @throws
UnreachableCode@8:4: Unreachable code detected
