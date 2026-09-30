===description===
new with array variable should error
===config===
suppress=MissingReturnType
===file===
<?php
function test(array $config) {
    new $config();
//      ^^^^^^^ InvalidStringClass: Dynamic class instantiation requires string or class-string type, got 'array'
}
===expect===
