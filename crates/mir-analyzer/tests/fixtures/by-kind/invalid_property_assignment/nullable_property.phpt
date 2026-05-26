===description===
nullable property
===file===
<?php
class Foo {
    public ?string $name;
}

$f = new Foo();
$f->name = null;
===expect===
===ignore===
TODO
