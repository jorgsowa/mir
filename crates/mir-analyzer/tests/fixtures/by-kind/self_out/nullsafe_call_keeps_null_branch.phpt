===description===
@psalm-self-out through a nullsafe call (?->) must not drop the null branch
— the call never ran if the receiver was null, so it's still possibly null
afterward.
===config===
suppress=UnusedParam
===file===
<?php
class A {
    /** @psalm-self-out Ready */
    public function touch(): void {}
}
class Ready extends A {
    public function commit(): void {}
}
function test(?A $x): void {
    $x?->touch();
    $x->commit();
}
===expect===
PossiblyNullMethodCall@11:4-11:16: Cannot call method commit() on possibly null value
