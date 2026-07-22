===description===
`$this<U>` is accepted as a synonym for `self<U>` in a self-out annotation,
and the `self`/`static`/`parent`/`$this` keyword match is case-insensitive
(PHP's own class-reference keywords are), same as the bare `self`/`static`
forms already are.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
final class Box
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @template U
     * @param U $value
     * @psalm-self-out $this<U>
     */
    public function replaceViaThis($value): void
    {
    }

    /**
     * @template U
     * @param U $value
     * @psalm-self-out SELF<U>
     */
    public function replaceViaUppercaseSelf($value): void
    {
    }
}

function test(): void {
    $box = new Box(1);
    $box->replaceViaThis("hello");
    /** @mir-check $box is Box<string> */
    $_ = 1;

    $other = new Box(1);
    $other->replaceViaUppercaseSelf(3.14);
    /** @mir-check $other is Box<float> */
    $_ = 1;
}
===expect===
