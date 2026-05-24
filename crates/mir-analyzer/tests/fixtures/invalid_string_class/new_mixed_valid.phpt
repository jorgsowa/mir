===description===
new with mixed variable should error - requires string or class-string type
===file===
<?php
function test(mixed $value) {
    new $value();
}
===expect===
InvalidStringClass@3:9: Dynamic class instantiation requires string or class-string type, got 'mixed'
