===description===
Assigning a property inside its own null guard narrows it for later reads,
including a property reached through one more receiver hop (`$this->cfg->p`).
Reassigning the intermediate property discards the stale narrowing.
===file===
<?php
class Cfg {
    public ?string $p = null;
}

class A {
    private ?string $p = null;
    private ?Cfg $cfg = null;
    private static ?string $sp = null;

    public function direct(): string {
        if ($this->p === null) { $this->p = 'x'; }
        $v = $this->p;
//      ^^ UnusedVariable: Variable $v is never read
        /** @mir-check $v is string */
        return $this->p;
    }

    public function bothBranches(): string {
        if ($this->p === null) {
            if (rand(0, 1)) { $this->p = 'x'; } else { $this->p = 'y'; }
        }
        return $this->p;
    }

    public function chained(): string {
        if ($this->cfg === null) { $this->cfg = new Cfg(); }
        if ($this->cfg->p === null) { $this->cfg->p = 'x'; }
        $v = $this->cfg->p;
//      ^^ UnusedVariable: Variable $v is never read
        /** @mir-check $v is string */
        return $this->cfg->p;
//      ^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
//      ^^^^^^^^^^^^^^^^^^^^^ TypeCheckMismatch: Type of $v is expected to be string, got string|null
    }

    public function intermediateReassigned(Cfg $other): string {
        if ($this->cfg === null) { $this->cfg = new Cfg(); }
        if ($this->cfg->p === null) { $this->cfg->p = 'x'; }
        $this->cfg = $other;
        return $this->cfg->p;
//      ^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function otherObject(A $o): string {
        if ($o->cfg === null) { $o->cfg = new Cfg(); }
        if ($o->cfg->p === null) { $o->cfg->p = 'x'; }
        return $o->cfg->p;
//      ^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function staticProp(): string {
        if (self::$sp === null) { self::$sp = 'x'; }
        return self::$sp;
    }

    public function notAssignedOnEveryPath(): string {
        if ($this->p === null && rand(0, 1)) { $this->p = 'x'; }
        return $this->p;
//      ^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }
}
===expect===
