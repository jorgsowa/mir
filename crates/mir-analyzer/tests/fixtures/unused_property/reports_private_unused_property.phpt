===source===
<?php
class Foo {
    private string $name = 'bar';
}
===expect===
UnusedProperty: Private property Foo::$name is never read
