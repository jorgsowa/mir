===description===
reports private unused property
===file===
<?php
class Foo {
    private string $name = 'bar';
}
===expect===
UnusedProperty@3:4-3:32: Private property Foo::$name is never read
