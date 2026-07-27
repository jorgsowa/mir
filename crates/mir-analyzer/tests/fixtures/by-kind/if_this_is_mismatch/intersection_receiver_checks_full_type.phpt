===description===
@if-this-is checking a method resolved through an intersection-typed receiver
compared only the declaring class's own atom against the constraint, dropping
the sibling intersection members — so a constraint naming a part the
declaring class doesn't itself implement (only the local intersection
provides it) both false-positived a satisfying receiver and would have
false-negatived a receiver missing that part, since the comparison target
never carried the other parts either way.
===config===
suppress=UnusedParam
===file===
<?php
interface HasBar {
    public function bar(): void;
}
class Foo {
    /** @if-this-is Foo&HasBar */
    public function onlyWithBar(): void {}
}
function withBoth(Foo&HasBar $x): void {
    $x->onlyWithBar();
}
function withFooOnly(Foo $y): void {
    $y->onlyWithBar();
}
===expect===
IfThisIsMismatch@13:4-13:21: Cannot call Foo::onlyWithBar() — @if-this-is requires $this to be 'Foo&HasBar', but it is 'Foo'
