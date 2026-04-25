===file===
<?php
function test(int $mode): void {
    switch ($mode) {
        case 1:
            break;
            echo 'unreachable';
    }
}
===expect===
UnreachableCode: Unreachable code detected
