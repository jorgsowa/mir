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
UnreachableCode@8:5-8:24: Unreachable code detected
