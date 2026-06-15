===description===
Static interface call
===config===
suppress=MissingReturnType
===file===
<?php
interface Foo {
    public static function doFoo();
}

Foo::doFoo();
===expect===
UndefinedClass@6:0-6:3: Class Foo does not exist
