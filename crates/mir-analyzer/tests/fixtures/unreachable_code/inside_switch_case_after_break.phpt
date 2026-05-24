===description===
inside switch case after break
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
UnreachableCode@6:13: Unreachable code detected
