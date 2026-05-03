===description===
property type hint via use
===file===
<?php
use Vendor\Missing\Foo;
class Bar {
    public Foo $prop;
}
===expect===
UndefinedClass: Class Vendor\Missing\Foo does not exist
===ignore===
TODO
