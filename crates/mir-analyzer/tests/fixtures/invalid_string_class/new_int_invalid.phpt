===description===
new with int variable should error
===file===
<?php
function test(int $value) {
    new $value();
}
===expect===
InvalidStringClass@3:8: Dynamic class instantiation requires string or class-string type, got 'int'
