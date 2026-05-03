===description===
variadic reported
===file===
<?php
function sum(int ...$nums): int {
    return 0;
}
===expect===
UnusedParam: Parameter $nums is never used
===ignore===
TODO
