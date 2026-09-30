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
//<^^^ UndefinedClass: Class Foo does not exist
===expect===
