===description===
A mutable (non-readonly, non-enum) object passed into an impure constructor from
an immutable context MUST still be flagged: only native-readonly classes and enums
are mutation-free by construction. This is the positive control guarding against
over-suppression of case #2.
===config===
suppress=MissingConstructor,InvalidReturnType
===file===
<?php
class MutableState {
    public array $items = [];
}

/** @implements \Iterator */
final class StateIterator implements \Iterator {
    private MutableState $source;

    public function __construct(MutableState $source) {
        $this->source = $source;
    }

    #[\ReturnTypeWillChange] public function current() {}
    #[\ReturnTypeWillChange] public function next(): void {}

    /** @psalm-mutation-free */
    #[\ReturnTypeWillChange]
    public function key() {}

    #[\ReturnTypeWillChange] public function valid(): bool { return true; }
    #[\ReturnTypeWillChange] public function rewind(): void {}
}

/** @psalm-immutable */
final class Holder {
    private MutableState $state;

    public function __construct(MutableState $state) {
        $this->state = $state;
    }

    /** @psalm-mutation-free */
    #[\ReturnTypeWillChange]
    public function iter(): StateIterator {
        return new StateIterator($this->state);
    }
}
===expect===
ImpureFunctionCall@36:15-36:46: Calling impure function StateIterator::__construct() in a @pure function
