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
NonStaticSelfCall
===ignore===
TODO
