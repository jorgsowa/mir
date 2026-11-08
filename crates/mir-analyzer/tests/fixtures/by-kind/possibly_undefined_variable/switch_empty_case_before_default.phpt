===description===
An empty `case` falls into the next case, so it doesn't leave the switch with the pre-switch state.
===file===
<?php
function f(int $t): string {
    switch ($t) {
        case 1: $m = 'one'; break;
        case 2:
        default: $m = 'other';
    }
    return $m;
}
