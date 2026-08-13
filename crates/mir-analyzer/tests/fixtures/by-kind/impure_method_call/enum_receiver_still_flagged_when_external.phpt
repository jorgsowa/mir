===description===
An enum receiver's method call from an immutable context is NOT flagged, but a
method on a NON-enum object (a mutable third-party class) stored in the same
immutable class still IS — only enumerations are mutation-free by construction.
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
