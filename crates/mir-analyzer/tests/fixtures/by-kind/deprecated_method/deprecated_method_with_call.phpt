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
//<^^^^^^^^^^^^^ DeprecatedMethodCall: Call to deprecated method Foo::barBar
===expect===
