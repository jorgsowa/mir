===description===
FP-K4: implementing a trait's ABSTRACT method with a narrower visibility is
legal PHP (verified live against the real binary) — unlike the same abstract
method declared on an interface or an abstract class, where narrowing is a
fatal error. `all_parent_methods` already excludes a directly-used trait's
CONCRETE methods from override checks (they're not "parents", they're
flattened into `own`), but an abstract trait method still went through the
normal never-narrower visibility check.
===config===
suppress=MissingConstructor
===file===
<?php
trait PublicRequirement {
    abstract public function foo(): void;
}

class NarrowsToProtected {
    use PublicRequirement;
    protected function foo(): void {}
}

trait ProtectedRequirement {
    abstract protected function bar(): void;
}

class NarrowsToPrivate {
    use ProtectedRequirement;
    private function bar(): void {}
}
===expect===
