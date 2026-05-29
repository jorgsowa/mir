===description===
variable-variable should mark operand as read, but other vars should still be reported as unused

===file===
<?php
function test() {
    $unused = 'never_used';
    $key = 'value';
    echo $$key;
}
===expect===
UnusedVariable@3:5-3:12: Variable $unused is never read
