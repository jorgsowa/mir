===description===
Regression (laravel/framework): a property with only a docblock @var (no native
type) accepts null at runtime. mir now widens an untyped (docblock-only) property
with null for the assignment check, so assigning a nullable value no longer emits
InvalidPropertyAssignment.
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
