===description===
Laravel FP (laravel/framework): a property with only a docblock @var (no native
type) accepts null at runtime, but mir treats the @var as a strict native type and
emits InvalidPropertyAssignment when a nullable value is assigned. Ignored pending
fix — see ROADMAP §1.4 (docblock @var treated as native type).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedProperty,MixedAssignment
===file===
<?php
class Connection {
    /** @var int */
    protected $ttl;

    public function __construct(?int $ttl) {
        $this->ttl = $ttl;
    }
}
===expect===
