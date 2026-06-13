===description===
ImpureStaticVariable does NOT fire inside a function that is NOT marked @pure.
===config===
suppress=UnusedVariable
===file===
<?php
function impure(): int {
    static $count = 0;
    return ++$count;
}

===expect===
