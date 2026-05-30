===description===
Deprecated method with call
===file===
<?php
class Foo {
    /**
     * @deprecated
     */
    public static function barBar(): void {
    }
}

Foo::barBar();
===expect===
DeprecatedMethodCall@10:1-10:14: Call to deprecated method Foo::barBar
