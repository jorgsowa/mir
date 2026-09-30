===description===
UnhandledMatchCondition fires when a match on a string literal union misses a case.
===file===
<?php
/** @param "red"|"green"|"blue" $color */
function label(string $color): string {
    return match($color) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: "blue"
        "red"   => "Red",
        "green" => "Green",
    };
}
===expect===
