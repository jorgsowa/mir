===description===
No crash with empty match
===file===
<?php
function foo(int $i) {
    match ($i) {

    };
}
===expect===
UnhandledMatchCondition
===ignore===
TODO
