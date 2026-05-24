===description===
deprecatedMethodWithCall
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
DeprecatedMethod
===ignore===
TODO
