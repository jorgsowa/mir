===description===
static call with int variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(int $value) {
    $value::method();
}
===expect===
InvalidStringClass@3:4-3:10: Dynamic class instantiation requires string or class-string type, got 'int'
