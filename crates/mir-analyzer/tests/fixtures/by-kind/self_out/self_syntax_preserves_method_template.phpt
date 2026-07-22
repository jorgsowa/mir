===description===
`@psalm-self-out self<U>` where `U` is the *method's own* `@template` (distinct
from the class's own template `T`) must substitute `U` with this call's
inferred binding — not erase the annotation's `<U>` and reattach the
receiver's pre-call type params, which would leave the receiver's stale
class-level binding untouched.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T
 */
final class MutableBox
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @template U
     * @param U $value
     * @psalm-self-out self<U>
     */
    public function replace($value): void
    {
    }
}

function test(): void {
    $box = new MutableBox(1);
    /** @mir-check $box is MutableBox<int> */
    $_ = 1;

    $box->replace("hello");
    /** @mir-check $box is MutableBox<string> */
    $_ = 1;
}
===expect===
