===description===
new with array variable should error
===file===
<?php
function test(array $config) {
    new $config();
}
===expect===
InvalidStringClass@3:8: Dynamic class instantiation requires string or class-string type, got 'array<mixed, mixed>'
