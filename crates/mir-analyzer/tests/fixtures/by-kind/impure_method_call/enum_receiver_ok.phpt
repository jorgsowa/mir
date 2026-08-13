===description===
Calling a method on an enum receiver from within an immutable context must not be
flagged as ImpureMethodCall: enumerations have no mutable properties, so every
enum instance method is mutation-free by construction — storing an enum in an
immutable (readonly) class and calling its methods cannot mutate state.
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
