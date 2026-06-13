===description===
does not report do while control var
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
