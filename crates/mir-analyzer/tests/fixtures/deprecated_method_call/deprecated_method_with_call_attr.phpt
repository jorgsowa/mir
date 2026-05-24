===description===
deprecatedMethodWithCallAttr
===file===
<?php
class Foo {
    #[\Deprecated]
    public static function barBar(): void {
    }
}

Foo::barBar();
===expect===
DeprecatedMethod
===ignore===
TODO
