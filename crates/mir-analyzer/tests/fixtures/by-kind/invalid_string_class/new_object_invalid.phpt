===description===
new with object variable should error
===config===
suppress=MissingReturnType
===file===
<?php
class Foo {}

function test(Foo $obj) {
    new $obj();
}
===expect===
InvalidStringClass@5:8-5:12: Dynamic class instantiation requires string or class-string type, got 'Foo'
