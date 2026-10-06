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
//  ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
