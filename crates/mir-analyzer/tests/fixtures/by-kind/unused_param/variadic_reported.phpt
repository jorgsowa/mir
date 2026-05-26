===description===
variadic reported
===file===
<?php
function sum(int ...$nums): int {
    return 0;
}
===expect===
UnusedParam@2:14: Parameter $nums is never used
