===description===
No crash with empty match
===config===
suppress=MissingReturnType
===file===
<?php
function foo(int $i) {
    match ($i) {
//  ^ +2:5 UnhandledMatchCondition: Unhandled match condition: no arms

    };
}
===expect===
