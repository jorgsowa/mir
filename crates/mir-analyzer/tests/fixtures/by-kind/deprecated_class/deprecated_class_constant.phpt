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
===expect===
DeprecatedClass@9:6-9:9: Class Foo is deprecated
