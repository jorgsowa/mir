===description===
UnhandledMatchCondition does NOT fire when all string literal union cases are covered.
===file===
<?php
/** @param "a"|"b"|"c" $s */
function label(string $s): string {
    return match($s) {
        "a" => "A",
        "b" => "B",
        "c" => "C",
    };
}
===expect===
