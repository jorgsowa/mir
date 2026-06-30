===description===
DeprecatedMethod fires when calling a deprecated parent method on a child instance that does not override it.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /** @deprecated use newMethod() instead */
    public function oldMethod(): void {}
}
class Child extends Base {}

function test(Child $c): void {
    $c->oldMethod();
}
===expect===
DeprecatedMethod@9:4-9:19: Method Child::oldMethod() is deprecated: use newMethod() instead
