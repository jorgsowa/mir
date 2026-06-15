===description===
parameter not reported as unused variable
===file===
<?php
function foo(int $param): int {
    return 42;
}
===expect===
UnusedParam@2:13-2:23: Parameter $param is never used
