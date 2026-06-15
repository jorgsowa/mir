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
DeprecatedMethodCall@10:0-10:13: Call to deprecated method Foo::barBar
