===description===
A memoized call chain narrowing is dropped when the receiver may have changed:
an impure call on it, a reassignment, or an unguarded sibling method.
===config===
memoize_method_call_results=true
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
class Identity {
    public ?int $accountId = null;
}
class Context {
    public function getIdentity(): Identity { return new Identity(); }
    public function reset(): void {}

    public function afterImpureCall(): int {
        if ($this->getIdentity()->accountId === null) {
            return 0;
        }
        $this->reset();
        return $this->getIdentity()->accountId;
    }

    public function afterOwnCallsOnly(): int {
        if ($this->getIdentity()->accountId === null) {
            return 0;
        }
        $this->getIdentity();
        return $this->getIdentity()->accountId;
    }
}

function afterReassignment(Context $ctx, Context $other): int {
    if ($ctx->getIdentity()->accountId === null) {
        return 0;
    }
    $ctx = $other;
    return $ctx->getIdentity()->accountId;
}
===expect===
NullableReturnStatement@14:8-14:47: Return type 'int|null' is not compatible with declared 'int'
NullableReturnStatement@31:4-31:42: Return type 'int|null' is not compatible with declared 'int'
