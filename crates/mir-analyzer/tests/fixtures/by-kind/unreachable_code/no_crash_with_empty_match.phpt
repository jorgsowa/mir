===description===
No crash with empty match
===file===
<?php
function foo(int $i) {
    match ($i) {

    };
}
===expect===
UnhandledMatchCondition@3:5-5:6: Unhandled match condition: no arms
