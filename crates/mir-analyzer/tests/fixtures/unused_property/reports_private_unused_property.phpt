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
UnusedProperty@1:0: Private property Foo::$name is never read
===ignore===
TODO
