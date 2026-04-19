===source===
<?php
// Variable assigned before the switch is always defined post-switch, even when
// some cases fall through without a break. Before the fallthrough-context fix,
// the fallthrough case's assignment was dropped, but the pre-existing $y still
// kept the function valid — no regression here.
function foo(int $x): string {
    $y = "default";
    switch ($x) {
        case 1:
            $y = "one";
            break;
        case 2:
            $y = "two";
            // no break — falls through to end of switch
    }
    return $y;
}
===expect===
