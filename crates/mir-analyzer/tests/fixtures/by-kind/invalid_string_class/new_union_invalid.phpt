===description===
new with union type containing non-string should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(int|bool $value) {
    new $value();
}
===expect===
InvalidStringClass@3:9-3:15: Dynamic class instantiation requires string or class-string type, got 'int|bool'
