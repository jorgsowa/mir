===description===
new with bool variable should error
===file===
<?php
function test(bool $flag) {
    new $flag();
}
===expect===
InvalidStringClass@3:9-3:14: Dynamic class instantiation requires string or class-string type, got 'bool'
