===description===
reports missing instance method
===file===
<?php
class Foo {}
function test(): void {
    $f = new Foo();
    $f->missing();
}
===expect===
UndefinedMethod@5:4: Method Foo::missing() does not exist
===ignore===
TODO
