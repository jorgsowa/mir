===description===
new with int variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(int $value) {
    new $value();
//      ^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'int'
}
===expect===
