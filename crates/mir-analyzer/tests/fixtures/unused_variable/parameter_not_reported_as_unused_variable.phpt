===source===
<?php
function foo(int $param): int {
    return 42;
}
===expect===
UnusedParam: $param
