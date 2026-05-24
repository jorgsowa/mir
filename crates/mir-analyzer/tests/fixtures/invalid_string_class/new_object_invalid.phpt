===description===
new with object variable should error
===file===
<?php
class Foo {}

function test(Foo $obj) {
    new $obj();
}
===expect===
InvalidStringClass@5:8: Dynamic class instantiation requires string or class-string type, got 'Foo'
