===file===
<?php
class Foo {
    /** @deprecated use newMethod() instead */
    public function oldMethod(): void {}
}

function test(Foo $foo): void {
    $foo->oldMethod();
}
===expect===
DeprecatedMethodCall: Call to deprecated method Foo::oldMethod
