===description===
Passing $this (under @psalm-immutable) or a parameter (under
@psalm-external-mutation-free) into a free-function call was never
checked — only @pure gated free-function calls at all; `new X(...)` and
method calls already had the identical check for the same object-argument
shape.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mutate() in a @pure function
    }
}

class Wrapper {
    /** @psalm-external-mutation-free */
    public function corruptParam(Box $box): void {
        mutate($box);
//      ^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mutate() in a @pure function
    }

    /** @psalm-external-mutation-free */
    public function safeValue(): void {
        mutate(new Box());
    }
}
