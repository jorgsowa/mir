===description===
MissingConstructor fires for a subclass that inherits a non-nullable uninitialized
property and adds no constructor of its own (nor does its parent provide one).
===file===
<?php
class Base {
//<^^^^^^^^^^^^ MissingConstructor: Class Base has uninitialized properties but no constructor
    public string $name;
}

class Child extends Base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class Child has uninitialized properties but no constructor

new Child();
