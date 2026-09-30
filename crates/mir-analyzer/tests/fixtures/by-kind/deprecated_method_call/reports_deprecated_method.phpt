===description===
reports deprecated method
===file===
<?php
class Foo {
    /** @deprecated use newMethod() instead */
    public function oldMethod(): void {}
}

function test(Foo $foo): void {
    $foo->oldMethod();
//  ^^^^^^^^^^^^^^^^^ DeprecatedMethod: Method Foo::oldMethod() is deprecated: use newMethod() instead
}
===expect===
