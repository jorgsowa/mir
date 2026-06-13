===description===
MissingConstructor
===file===
<?php
class Foo {
    public string $name;
}

new Foo();

===expect===
MissingConstructor@2:0-2:11: Class Foo has uninitialized properties but no constructor
