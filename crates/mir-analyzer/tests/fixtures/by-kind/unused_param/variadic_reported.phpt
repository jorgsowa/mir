===description===
variadic reported
===file===
<?php
function sum(int ...$nums): int {
//           ^^^^^^^^^^^^ UnusedParam: Parameter $nums is never used
    return 0;
}
