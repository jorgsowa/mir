===description===
inside switch case after break
===file===
<?php
function test(int $mode): void {
    switch ($mode) {
        case 1:
            break;
            echo 'unreachable';
//          ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }
}
===expect===
