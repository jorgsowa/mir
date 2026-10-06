===description===
reports missing instance method
===file===
<?php
class Foo {}
function test(): void {
    $f = new Foo();
    $f->missing();
//  ^^^^^^^^^^^^^ UndefinedMethod: Method Foo::missing() does not exist
}
