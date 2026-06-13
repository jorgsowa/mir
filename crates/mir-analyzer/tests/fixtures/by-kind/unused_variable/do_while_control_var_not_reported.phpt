===description===
do while control var not reported
===config===
suppress=UnusedVariable
===file===
<?php
function foo(): void {
    do {
        $run = false;
        if (time() % 3 === 0) {
            continue;
        }
        $run = true;
    } while ($run);
}
===expect===
