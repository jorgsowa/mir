===description===
The receiver-scoped invalidation isn't special-cased to `$this` — calling an
unproven method on ANY narrowed object variable must drop that object's own
narrowing, since the callee's `$this` is that object.
===config===
suppress=UnusedVariable
===file===
<?php
class Box {
    public ?string $value = null;

    public function reset(): void {
        if (rand(0, 1) === 1) {
            $this->value = null;
        }
    }
}

function test(Box $other): void {
    $other->value = 'set';
    /** @mir-check $other->value is string */
    $_ = 1;
    $other->reset();
    /** @mir-check $other->value is string|null */
    $_ = 1;
}
===expect===
