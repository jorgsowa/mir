===description===
parameter not reported as unused variable
===file===
<?php
function foo(int $param): int {
    return 42;
}
===expect===
UnusedParam: Parameter $param is never used
===ignore===
TODO
