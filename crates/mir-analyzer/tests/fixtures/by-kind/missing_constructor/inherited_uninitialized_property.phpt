===description===
MissingConstructor fires for a subclass that inherits a non-nullable uninitialized
property and adds no constructor of its own (nor does its parent provide one).
===file===
<?php
class Base {
    public string $name;
}

class Child extends Base {}

new Child();

===expect===
MissingConstructor@2:0-2:12: Class Base has uninitialized properties but no constructor
MissingConstructor@6:0-6:27: Class Child has uninitialized properties but no constructor
