===description===
property type hint via use
===file===
<?php
use Vendor\Missing\Foo;
class Bar {
    public Foo $prop;
}
===expect===
MissingConstructor@3:0-3:11: Class Bar has uninitialized properties but no constructor
UndefinedClass@4:12-4:15: Class Vendor\Missing\Foo does not exist
