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
}
===expect===
DeprecatedMethod@8:4-8:19: Method Foo::oldMethod() is deprecated
