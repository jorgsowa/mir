===description===
noCrashWithEmptyMatch
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
