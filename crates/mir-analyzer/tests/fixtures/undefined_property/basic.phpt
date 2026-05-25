===description===
Basic
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
UndefinedProperty@7:14: Property Foo::$nonexistent does not exist
