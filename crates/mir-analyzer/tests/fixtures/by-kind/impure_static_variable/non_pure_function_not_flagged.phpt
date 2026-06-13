===description===
ImpureStaticVariable does NOT fire inside a function that is NOT marked @pure.
===file===
<?php
function impure(): int {
    static $count = 0;
    return ++$count;
}

===expect===
