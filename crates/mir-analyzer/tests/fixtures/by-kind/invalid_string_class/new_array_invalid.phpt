===description===
new with array variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(array $config) {
    new $config();
}
===expect===
InvalidStringClass@3:8-3:15: Dynamic class instantiation requires string or class-string type, got 'array'
