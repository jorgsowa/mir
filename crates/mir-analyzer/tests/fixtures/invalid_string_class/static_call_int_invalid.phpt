===description===
static call with int variable should error
===file===
<?php
function test(int $value) {
    $value::method();
}
===expect===
InvalidStringClass@3:5: Dynamic class instantiation requires string or class-string type, got 'int'
