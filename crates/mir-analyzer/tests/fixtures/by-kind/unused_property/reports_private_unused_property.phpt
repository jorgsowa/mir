===description===
reports private unused property
===file===
<?php
class Foo {
//<^ +2:1 UnusedClass: Class Foo is never referenced
    private string $name = 'bar';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Foo::$name is never read
}
