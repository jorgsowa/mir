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
UnreachableCode: Unreachable code detected
