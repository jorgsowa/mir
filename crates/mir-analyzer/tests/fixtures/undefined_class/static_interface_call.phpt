===description===
staticInterfaceCall
===file===
<?php
interface Foo {
    public static function doFoo();
}

Foo::doFoo();
===expect===
UndefinedClass@6:0: Class Foo does not exist
