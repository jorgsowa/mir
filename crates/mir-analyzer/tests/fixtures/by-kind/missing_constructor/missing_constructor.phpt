===description===
MissingConstructor
===file===
<?php
class Foo {
//<^^^^^^^^^^^ MissingConstructor: Class Foo has uninitialized properties but no constructor
    public string $name;
}

new Foo();

===expect===
