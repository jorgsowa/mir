===description===
Mutable object method calls remain impure in immutable contexts.
===config===
suppress=MissingConstructor
===file===
<?php
class MutableCode {
    public function mutates(): void {}
}

/** @psalm-immutable */
final readonly class IssueTriggerPositive {
    private ?MutableCode $callee;

    public function __construct(?MutableCode $callee) {
        $this->callee = $callee;
    }

    /** Calling a method that could mutate an external object must still be flagged. */
    public function isSelf(): bool {
        return $this->callee !== null && $this->callee->mutates();
    }
}

enum Code: string {
    case FirstParty = 'first-party';

    public function isFirstPartyOrTest(): bool {
        return $this === self::FirstParty;
    }
}
===expect===
ImpureMethodCall@16:41-16:65: Calling impure method mutates() in a pure or immutable context
