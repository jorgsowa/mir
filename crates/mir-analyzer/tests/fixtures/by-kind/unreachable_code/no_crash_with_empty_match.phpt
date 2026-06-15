===description===
No crash with empty match
===config===
suppress=MissingReturnType
===file===
<?php
function foo(int $i) {
    match ($i) {

    };
}
===expect===
UnhandledMatchCondition@3:4-5:5: Unhandled match condition: no arms
