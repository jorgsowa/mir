===description===
property type hint via use
===file===
<?php
use Vendor\Missing\Foo;
class Bar {
    public Foo $prop;
}
===expect===
UndefinedClass@4:12: Class Vendor\Missing\Foo does not exist
