===description===
new with bool variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(bool $flag) {
    new $flag();
//      ^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'bool'
}
===expect===
