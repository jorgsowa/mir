===description===
Static interface call
===file===
<?php
interface Foo {
    public static function doFoo();
}

Foo::doFoo();
===expect===
UndefinedClass@6:1-6:4: Class Foo does not exist
