===source===
<?php
class Foo {
    public string $name = '';
}
function test(): void {
    $f = new Foo();
    echo $f->nonexistent;
}
===expect===
UndefinedProperty: $f->nonexistent
