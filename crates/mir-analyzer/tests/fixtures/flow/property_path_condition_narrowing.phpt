===description===
Truthiness and null checks on a two-hop property path (`$this->cfg->domain`) narrow it in `&&` chains, ternaries and guards. A call on the receiver or a write to an intermediate property discards the narrowing.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Cfg {
    public bool $isLocal = false;
    public ?string $domain = null;
    public ?Cfg $next = null;
    public function reset(): void {}
}

class A {
    private Cfg $cfg;
    private ?Cfg $maybe = null;

    public function andTernary(): string {
        if ($this->cfg->isLocal && $this->cfg->domain) {
            $v = $this->cfg->domain;
            /** @mir-check $v is non-empty-string */
            return $v;
        }
        return $this->cfg->isLocal && $this->cfg->domain ? $this->cfg->domain : 'default';
    }

    public function notNullInAnd(): string {
        return $this->cfg->isLocal && $this->cfg->domain !== null ? $this->cfg->domain : 'default';
    }

    public function looseNotNull(): string {
        return $this->cfg->domain != null ? $this->cfg->domain : 'default';
    }

    public function guardReturn(): string {
        if ($this->cfg->domain === null) {
            return 'default';
        }
        $v = $this->cfg->domain;
        /** @mir-check $v is string */
        return $v;
    }

    public function falsyKeepsNull(): ?string {
        if (!$this->cfg->domain) {
            $v = $this->cfg->domain;
            /** @mir-check $v is string|null */
            return $v;
        }
        return null;
    }

    public function nullBranch(): ?string {
        if ($this->cfg->domain === null) {
            $v = $this->cfg->domain;
            /** @mir-check $v is null */
            return $v;
        }
        return null;
    }

    public function mergedBranches(bool $c): ?string {
        $v = $c && $this->cfg->domain ? $this->cfg->domain : null;
        /** @mir-check $v is non-empty-string|null */
        return $v;
    }

    public function callOnReceiverInvalidates(): string {
        if ($this->cfg->domain === null) {
            return 'default';
        }
        $this->cfg->reset();
        return $this->cfg->domain;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function callOnThisInvalidates(): string {
        if ($this->cfg->domain === null) {
            return 'default';
        }
        $this->touch();
        return $this->cfg->domain;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function intermediateWriteInvalidates(Cfg $other): string {
        if ($this->cfg->domain === null) {
            return 'default';
        }
        $this->cfg = $other;
        return $this->cfg->domain;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function leafWriteReplacesNarrowing(): string {
        if ($this->cfg->domain === null) {
            return 'default';
        }
        $this->cfg->domain = null;
        return $this->cfg->domain;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'null' is not compatible with declared 'string'
    }

    public function nullableIntermediateStaysNullable(): ?string {
        if ($this->maybe?->domain) {
            $m = $this->maybe;
            /** @mir-check $m is Cfg */
            return $m->domain;
        }
        return null;
    }

    private function touch(): void {}
}
