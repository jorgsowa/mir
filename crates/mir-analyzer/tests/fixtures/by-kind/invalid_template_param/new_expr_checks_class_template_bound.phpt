===description===
G1: a class-level `@template T of Bound` must be enforced on `new`
(constructor-argument inference), not just at method-call sites — this is
the dominant real-world generics workflow and was previously never checked.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
class Base {}
class Unrelated extends Base {}
class NotBase {}

/** @template T of Base */
class Box {
    /** @param T $item */
    public function __construct(private $item) {}
}

// Satisfies the bound (Unrelated extends Base) — no error.
$ok = new Box(new Unrelated());

// Violates the bound — NotBase does not extend Base.
$bad = new Box(new NotBase());
===expect===
InvalidTemplateParam@16:7-16:29: Template type 'T' inferred as 'NotBase' does not satisfy bound 'Base'
