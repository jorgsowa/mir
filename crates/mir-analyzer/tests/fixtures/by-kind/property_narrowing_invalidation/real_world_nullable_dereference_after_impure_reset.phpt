===description===
End-to-end regression for the motivating bug: `$this->user` was narrowed
non-null by `login()`'s own assignment, then `reset()` (an unproven method
of the same class) could null it back out — before the fix, mir kept
treating `$this->user` as non-null after the call and missed the possibly-
null dereference on `getId()`.
===file===
<?php
class UserRef {
    public function getId(): int { return 1; }
}
class Session {
    private ?UserRef $user = null;

    public function login(UserRef $u): void {
        $this->user = $u;
        $this->reset();
        $this->user->getId();
    }

    private function reset(): void {
        if (rand(0, 1) === 1) {
            $this->user = null;
        }
    }
}
===expect===
PossiblyNullMethodCall@11:8-11:28: Cannot call method getId() on possibly null value
