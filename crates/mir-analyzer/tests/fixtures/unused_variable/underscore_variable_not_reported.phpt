===description===
Variables whose names start with _ are treated as intentionally unused and
are not reported as UnusedVariable.
===file===
<?php
function foo(): int {
    $_ignored = 1;
    return 42;
}
===expect===
===ignore===
TODO
