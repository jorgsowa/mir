===description===
UnhandledMatchCondition fires when a match on a string literal union misses a case.
===file===
<?php
/** @param "red"|"green"|"blue" $color */
function label(string $color): string {
    return match($color) {
        "red"   => "Red",
        "green" => "Green",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:12: Unhandled match condition: "blue"
