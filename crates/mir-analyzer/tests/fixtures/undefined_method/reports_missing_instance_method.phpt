===source===
<?php
class Foo {}
function test(): void {
    $f = new Foo();
    $f->missing();
}
===expect===
UndefinedMethod at 5:4
