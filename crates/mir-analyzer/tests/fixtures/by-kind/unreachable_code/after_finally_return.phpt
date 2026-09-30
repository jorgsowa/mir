===description===
after finally return
===file===
<?php
function test(): void {
    try {
        echo 'work';
    } finally {
        return;
    }

    echo 'unreachable';
//  ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
