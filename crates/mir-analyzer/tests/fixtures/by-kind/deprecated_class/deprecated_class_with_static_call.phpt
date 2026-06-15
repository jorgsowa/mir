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
DeprecatedClass@10:0-10:3: Class Foo is deprecated
