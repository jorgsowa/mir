===file===
<?php
function foo(int $param): int {
    return 42;
}
===expect===
UnusedParam: Parameter $param is never used
