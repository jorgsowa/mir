===description===
False-positive guards for constructor definite-assignment: a promoted
property is implicitly assigned; a property assigned in EVERY branch of an
if/else is definite; a property assigned only after an early-return guard
clause (the other branch throws) is definite on the reachable path; a
property with a default value, a nullable property, and a docblock-only
(non-native) property never require initialization at all.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
class PromotedOk {
    public function __construct(public string $name) {}
}

class BothBranchesOk {
    public string $value;
    public function __construct(bool $cond, string $v) {
        if ($cond) {
            $this->value = $v;
        } else {
            $this->value = $v . '!';
        }
    }
}

class GuardClauseOk {
    public string $value;
    public function __construct(?string $v) {
        if ($v === null) {
            throw new \InvalidArgumentException('v required');
        }
        $this->value = $v;
    }
}

class DefaultValueOk {
    public string $value = 'default';
    public function __construct() {}
}

class NullableOk {
    public ?string $value;
    public function __construct() {}
}

class DocblockOnlyOk {
    /** @var string */
    public $value;
    public function __construct() {}
}
===expect===
