===description===
A default arm suppresses UnhandledMatchCondition even for int literal unions.
===file===
<?php
/** @param 1|2|3 $n */
function label(int $n): string {
    return match($n) {
        1 => "one",
        default => "other",
    };
}
===expect===
