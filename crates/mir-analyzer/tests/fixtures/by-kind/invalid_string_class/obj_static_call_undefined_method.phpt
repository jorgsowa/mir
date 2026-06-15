===description===
$obj::undefinedMethod() where $obj is a typed object should still emit UndefinedMethod
===file===
<?php
class Foo {}

function test(Foo $obj): void {
    $obj::nonExistent();
}
===expect===
UndefinedMethod@5:4-5:23: Method Foo::nonExistent() does not exist
