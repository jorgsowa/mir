===description===
Deprecated class constant
===file===
<?php
/**
 * @deprecated
 */
class Foo {
    public const FOO = 5;
}

echo Foo::FOO;
//   ^^^ DeprecatedClass: Class Foo is deprecated
