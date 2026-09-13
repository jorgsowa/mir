===description===
Enum method calls are allowed in immutable contexts.
===config===
suppress=MissingConstructor
===file===
<?php
/** @psalm-immutable */
final readonly class IssueTrigger {
    private ?Code $callee;

    public function __construct(?Code $callee) {
        $this->callee = $callee;
    }

    /** An issue is triggered in first-party code or test code. */
    public function isSelf(): bool {
        return $this->callee !== null && $this->callee->isFirstPartyOrTest();
    }
}

enum Code: string {
    case FirstParty = 'first-party';
    case Test       = 'test';

    public function isFirstPartyOrTest(): bool {
        return $this === self::FirstParty || $this === self::Test;
    }
}
===expect===
