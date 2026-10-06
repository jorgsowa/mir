===description===
reports private unused property
===file===
<?php
class Foo {
    private string $name = 'bar';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Foo::$name is never read
}
