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
InvalidStringClass@5:9-5:13: Dynamic class instantiation requires string or class-string type, got 'Foo'
