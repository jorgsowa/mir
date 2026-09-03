===description===
A `readonly` property narrowed via a `!== null`/`=== null` guard before a
closure or arrow function literal must keep that narrowing inside the
closure body — the value can never change after construction, so the guard
stays valid no matter when the closure runs. analyze_closure/
analyze_arrow_function built a fresh FlowState and copied purity flags,
template param names, etc., but never carried over ctx.prop_refined, so
even a readonly property's proven-non-null narrowing was discarded.
===file===
<?php
class Box {
    public function __construct(public readonly ?string $value) {}

    public function toClosure(): \Closure {
        if ($this->value === null) {
            return fn(): int => 0;
        }
        return function (): int {
            return strlen($this->value);
        };
    }

    public function toArrow(): \Closure {
        if ($this->value === null) {
            return fn(): int => 0;
        }
        return fn(): int => strlen($this->value);
    }
}

class MutableBox {
    public ?string $value = null;

    public function toClosure(): \Closure {
        if ($this->value === null) {
            return fn(): int => 0;
        }
        // Negative control: an ordinary (non-readonly) property could still
        // change before this closure runs, so its narrowing must NOT survive.
        return function (): int {
            return strlen($this->value);
        };
    }
}
===expect===
PossiblyNullArgument@32:26-32:38: Argument $string of strlen() might be null
