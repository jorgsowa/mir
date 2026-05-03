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
}
===expect===
UnreachableCode@9:4: Unreachable code detected
===ignore===
TODO
