===description===
reports private unused property
===config===
find_dead_code=true
===file===
<?php
class Foo {
    private string $name = 'bar';
}
===expect===
UnusedProperty@3:4: Private property Foo::$name is never read
