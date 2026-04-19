===source===
<?php
class Foo {}
function test(): void {
    $f = new Foo();
    $f->missing();
}
===expect===
UndefinedMethod: Method Foo::missing() does not exist
