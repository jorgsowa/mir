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
===expect===
DeprecatedClass@10:1-10:4: Class Foo is deprecated
