===description===
Narrowing must survive a call to a method proven not to touch `$this`
(`@pure` or `@psalm-mutation-free`) — only calls we can't prove safe should
invalidate a previously narrowed property.
===config===
suppress=UnusedVariable
===file===
<?php
class Session {
    private ?string $user = null;

    public function withPureCall(string $u): void {
        $this->user = $u;
        $this->describe();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function withMutationFreeCall(string $u): void {
        $this->user = $u;
        $this->currentUser();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    /** @pure */
    public function describe(): string {
        return 'session';
    }

    /** @psalm-mutation-free */
    public function currentUser(): ?string {
        return $this->user;
    }
}
===expect===
