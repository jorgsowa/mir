===config===
find_dead_code=true
===file===
<?php
class Foo {
    private string $name = 'bar';
}
===expect===
UnusedProperty: Private property Foo::$name is never read
