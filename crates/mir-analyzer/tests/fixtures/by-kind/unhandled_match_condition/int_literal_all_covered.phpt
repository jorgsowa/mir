===description===
UnhandledMatchCondition does NOT fire when all int literal union cases are covered.
===file===
<?php
/** @param 1|2|3 $n */
function label(int $n): string {
    return match($n) {
        1 => "one",
        2 => "two",
        3 => "three",
    };
}
===expect===
