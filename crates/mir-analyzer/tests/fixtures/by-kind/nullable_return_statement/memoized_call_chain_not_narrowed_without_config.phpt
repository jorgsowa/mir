===description===
Without memoize_method_call_results an unannotated getter is not assumed
stable, so the guarded read keeps its nullable declared type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Identity {
    /** @var positive-int|null */
    public ?int $accountId = null;
}
class Context {
    public function getIdentity(): Identity { return new Identity(); }

    /** @return positive-int */
    public function viaThrow(): int {
        if (!$this->getIdentity()->accountId) {
            throw new \Exception('no account');
        }
        return $this->getIdentity()->accountId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'positive-int|null' is not compatible with declared 'positive-int'
    }
}
