===description===
Self non static invocation
===file===
<?php
class A {
    public function fooFoo(): void {}

    public static function barBar(): void {
        self::fooFoo();
//      ^^^^^^^^^^^^^^ NonStaticSelfCall: Non-static method A::fooFoo() cannot be called statically
    }
}
===expect===
