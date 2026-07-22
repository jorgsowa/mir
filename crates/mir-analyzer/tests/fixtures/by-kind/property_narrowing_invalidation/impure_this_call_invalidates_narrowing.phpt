===description===
A property narrowed by direct assignment (`$this->user = $u`) must not stay
narrowed across a call to another method of the same class that isn't proven
pure/mutation-free — the callee's `$this` is the same object and may
reassign the property (e.g. reset it back to null).
===config===
suppress=UnusedVariable
===file===
<?php
class Session {
    private ?string $user = null;

    public function login(string $u): void {
        $this->user = $u;
        /** @mir-check $this->user is string */
        $_ = 1;
        $this->reset();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    private function reset(): void {
        if (rand(0, 1) === 1) {
            $this->user = null;
        }
    }
}
===expect===
