===description===
Self non static invocation
===file===
<?php
class A {
    public function fooFoo(): void {}

    public static function barBar(): void {
        self::fooFoo();
    }
}
===expect===
NonStaticSelfCall@6:9-6:23: Non-static method A::fooFoo() cannot be called on self:: in a static context
