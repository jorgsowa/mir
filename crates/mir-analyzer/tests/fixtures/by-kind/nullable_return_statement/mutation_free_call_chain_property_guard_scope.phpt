===description===
The guard on a @mutation-free call chain is dropped when the receiver may
have changed, and never applies to calls with arguments or to a chain that
contains an unstable call.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Identity {
    public ?int $accountId = null;
}
class Context {
    private int $resets = 0;
    /** @psalm-mutation-free */
    public function getIdentity(): Identity { return new Identity(); }
    /** @psalm-mutation-free */
    public function byKey(string $key): Identity { return new Identity(); }
    public function unstable(): Context { return $this; }
    public function reset(): void { $this->resets++; }

    public function afterImpureCall(): int {
        if ($this->getIdentity()->accountId === null) {
            return 0;
        }
        $this->reset();
        return $this->getIdentity()->accountId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }

    public function withArgument(): int {
        if ($this->byKey('a')->accountId === null) {
            return 0;
        }
        return $this->byKey('a')->accountId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }

    public function throughUnstableHop(): int {
        if ($this->unstable()->getIdentity()->accountId === null) {
            return 0;
        }
        return $this->unstable()->getIdentity()->accountId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }
}

function afterReassignment(Context $ctx, Context $other): int {
    if ($ctx->getIdentity()->accountId === null) {
        return 0;
    }
    $ctx = $other;
    return $ctx->getIdentity()->accountId;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
}
