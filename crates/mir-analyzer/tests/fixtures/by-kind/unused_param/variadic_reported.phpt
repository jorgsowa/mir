===description===
variadic reported
===file===
<?php
function sum(int ...$nums): int {
    return 0;
}
===expect===
UnusedParam@2:13-2:25: Parameter $nums is never used
