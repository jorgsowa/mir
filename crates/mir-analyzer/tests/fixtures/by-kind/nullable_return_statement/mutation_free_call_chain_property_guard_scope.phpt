===description===
The guard on a @mutation-free call chain is dropped when the receiver may
have changed, and never applies to calls with arguments or to a chain that
contains an unstable call.
===config===
suppress=UnusedVariable,MissingThrowsDocblock,UnusedParam
===file===
<?php
class Identity {
    public ?int $accountId = null;
}
class Context {
    /** @psalm-mutation-free */
    public function getIdentity(): Identity { return new Identity(); }
    /** @psalm-mutation-free */
    public function byKey(string $key): Identity { return new Identity(); }
    public function unstable(): Context { return $this; }
    public function reset(): void {}

    public function afterImpureCall(): int {
        if ($this->getIdentity()->accountId === null) {
            return 0;
        }
        $this->reset();
        return $this->getIdentity()->accountId;
    }

    public function withArgument(): int {
        if ($this->byKey('a')->accountId === null) {
            return 0;
        }
        return $this->byKey('a')->accountId;
    }

    public function throughUnstableHop(): int {
        if ($this->unstable()->getIdentity()->accountId === null) {
            return 0;
        }
        return $this->unstable()->getIdentity()->accountId;
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
NullableReturnStatement@18:8-18:47: Return type 'int|null' is not compatible with declared 'int'
NullableReturnStatement@25:8-25:44: Return type 'int|null' is not compatible with declared 'int'
NullableReturnStatement@32:8-32:59: Return type 'int|null' is not compatible with declared 'int'
NullableReturnStatement@41:4-41:42: Return type 'int|null' is not compatible with declared 'int'
