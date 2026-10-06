===description===
property type hint via use
===file===
<?php
use Vendor\Missing\Foo;
class Bar {
//<^^^^^^^^^^^ MissingConstructor: Class Bar has uninitialized properties but no constructor
    public Foo $prop;
//         ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
}
