===description===
deprecatedClassConstant
===file===
<?php
/**
 * @deprecated
 */
class Foo {
    public const FOO = 5;
}

echo Foo::FOO;
===expect===
DeprecatedClass
===ignore===
TODO
