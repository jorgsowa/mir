===description===
Deprecated class with static call
===file===
<?php
/**
 * @deprecated
 */
class Foo {
    public static function barBar(): void {
    }
}

Foo::barBar();
//<^^^ DeprecatedClass: Class Foo is deprecated
===expect===
