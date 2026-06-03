===description===
Deprecated method with call attr
===file===
<?php
class Foo {
    #[Deprecated]
    public static function barBar(): void {
    }
}

Foo::barBar();
===expect===
DeprecatedMethodCall@8:1-8:14: Call to deprecated method Foo::barBar
