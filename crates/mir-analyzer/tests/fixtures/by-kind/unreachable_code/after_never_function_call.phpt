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
UnreachableCode@8:4-8:23: Unreachable code detected
