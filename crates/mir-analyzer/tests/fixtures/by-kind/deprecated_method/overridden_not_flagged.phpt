===description===
DeprecatedMethod does NOT fire when a child class overrides the parent's deprecated method without carrying @deprecated — the override itself is not deprecated.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /** @deprecated use newMethod() instead */
    public function oldMethod(): void {}
}
class Child extends Base {
    public function oldMethod(): void {}
}

function test(Child $c): void {
    $c->oldMethod();
}
===expect===
