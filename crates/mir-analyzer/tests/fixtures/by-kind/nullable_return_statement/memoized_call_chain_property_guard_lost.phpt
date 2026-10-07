===description===
A memoized call chain narrowing is dropped when the receiver may have changed:
an impure call on it, a reassignment, or an unguarded sibling method.
===config===
<mir memoizeMethodCallResults="true">
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Identity {
    public ?int $accountId = null;
}
class Context {
    private int $resets = 0;
    public function getIdentity(): Identity { return new Identity(); }
    public function reset(): void { $this->resets++; }

    public function afterImpureCall(): int {
        if ($this->getIdentity()->accountId === null) {
            return 0;
        }
        $this->reset();
        return $this->getIdentity()->accountId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
}
