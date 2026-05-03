===description===
basic
===file===
<?php
class Foo {
    public string $name = '';
}
function test(): void {
    $f = new Foo();
    echo $f->nonexistent;
}
===expect===
UndefinedProperty@7:13: Property Foo::$nonexistent does not exist
===ignore===
TODO
