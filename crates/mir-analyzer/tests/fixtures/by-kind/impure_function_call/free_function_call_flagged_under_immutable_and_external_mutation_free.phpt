===description===
Passing $this (under @psalm-immutable) or a parameter (under
@psalm-external-mutation-free) into a free-function call was never
checked — only @pure gated free-function calls at all; `new X(...)` and
method calls already had the identical check for the same object-argument
shape.
===config===
suppress=MissingConstructor,MixedArgument,MixedArrayAssignment
===file===
<?php
class Box {
    public array $items = [];
}

function mutate(Box $b): void {
    $b->items[] = 1;
}

/** @psalm-immutable */
class Holder {
    public Box $box;

    public function corruptThis(): void {
        mutate($this->box);
    }
}

class Wrapper {
    /** @psalm-external-mutation-free */
    public function corruptParam(Box $box): void {
        mutate($box);
    }

    /** @psalm-external-mutation-free */
    public function safeValue(): void {
        mutate(new Box());
    }
}
===expect===
ImpureFunctionCall@15:8-15:26: Calling impure function mutate() in a @pure function
ImpureFunctionCall@22:8-22:20: Calling impure function mutate() in a @pure function
