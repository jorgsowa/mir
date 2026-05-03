===description===
do while control var not reported
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
===ignore===
TODO
