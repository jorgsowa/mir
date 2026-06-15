===description===
Deprecated method with call attr
===file===
<?php
class Foo {
    #[\Deprecated]
    public static function barBar(): void {
    }
}

Foo::barBar();
===expect===
DeprecatedMethodCall@8:0-8:13: Call to deprecated method Foo::barBar
