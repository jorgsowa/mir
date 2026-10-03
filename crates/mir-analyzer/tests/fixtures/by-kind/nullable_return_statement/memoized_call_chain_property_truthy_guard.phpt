===description===
With memoize_method_call_results, a guard on `$this->getIdentity()->accountId`
narrows the next identical read, so the nullable declared type is not returned.
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
        $id = $this->getIdentity()->accountId;
        /** @mir-check $id is positive-int */
        return $id;
    }

    /** @return positive-int */
    public function viaNullCheck(): int {
        if ($this->getIdentity()->accountId === null) {
            throw new \Exception('no account');
        }
        return $this->getIdentity()->accountId;
    }

    /** @return positive-int */
    public function viaPositiveBranch(): int {
        if ($this->getIdentity()->accountId) {
            return $this->getIdentity()->accountId;
        }
        return 1;
    }
}

function onParameter(Context $ctx): int {
    if ($ctx->getIdentity()->accountId === null) {
        return 0;
    }
    $id = $ctx->getIdentity()->accountId;
    /** @mir-check $id is positive-int */
    return $id;
}
===expect===
