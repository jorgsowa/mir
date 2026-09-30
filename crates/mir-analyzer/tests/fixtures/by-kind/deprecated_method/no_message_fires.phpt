===description===
DeprecatedMethod fires without a trailing message when @deprecated has no text.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {
    /** @deprecated */
    public function oldMethod(): void {}
}

function test(Foo $f): void {
    $f->oldMethod();
//  ^^^^^^^^^^^^^^^ DeprecatedMethod: Method Foo::oldMethod() is deprecated
}
===expect===
