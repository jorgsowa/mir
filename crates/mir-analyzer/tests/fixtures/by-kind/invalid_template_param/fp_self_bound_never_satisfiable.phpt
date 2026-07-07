===description===
FP: a `@template T of self` bound stored the `self` bound atom with an empty
fqcn placeholder that was never filled in with the declaring class, so no
argument — not even an instance of the declaring class itself — could ever
satisfy the bound. A real violation on the same bound must still be caught.
===config===
suppress=UnusedVariable,MissingReturnType,UnusedParam
===file===
<?php
class Base {
    /**
     * @template T of self
     * @param T $x
     */
    public function accept($x): void {}
}
class Unrelated {}

$base = new Base();
$base->accept($base);
$base->accept(new Base());

$base->accept(new Unrelated());
===expect===
InvalidTemplateParam@15:0-15:30: Template type 'T' inferred as 'Unrelated' does not satisfy bound 'self(Base)'
