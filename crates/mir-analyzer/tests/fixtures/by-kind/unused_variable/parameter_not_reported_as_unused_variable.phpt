===description===
parameter not reported as unused variable
===file===
<?php
function foo(int $param): int {
//           ^^^^^^^^^^ UnusedParam: Parameter $param is never used
    return 42;
}
