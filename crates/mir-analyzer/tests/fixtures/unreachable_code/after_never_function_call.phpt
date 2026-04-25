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
UnreachableCode: Unreachable code detected
