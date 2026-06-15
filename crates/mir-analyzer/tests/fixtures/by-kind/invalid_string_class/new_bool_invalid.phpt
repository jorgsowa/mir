===description===
new with bool variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(bool $flag) {
    new $flag();
}
===expect===
InvalidStringClass@3:8-3:13: Dynamic class instantiation requires string or class-string type, got 'bool'
