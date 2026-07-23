===description===
A static call (self::/parent::) had no immutable-enforcement mirror of
method.rs's `$this->method()` check — only `is_pure` was ever checked for
a static call, so calling a non-mutation-free helper via self:: from
inside an immutable-class method silently bypassed the check that the
identical `$this->mutateHelper()` form already caught.
===file===
<?php
/** @psalm-immutable */
class C {
    public int $x = 0;

    public function f(): void {
        self::mutateHelper();
    }

    public function mutateHelper(): void {
        $this->x = 1;
    }
}
===expect===
ImpureMethodCall@7:8-7:28: Calling impure method mutateHelper() in a pure or immutable context
ImmutablePropertyModification@11:8-11:20: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
