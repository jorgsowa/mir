===description===
new with int variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(int $value) {
    new $value();
}
===expect===
InvalidStringClass@3:9-3:15: Dynamic class instantiation requires string or class-string type, got 'int'
