===description===
`@psalm-this-out` is a recognized alias for `@psalm-self-out` (Psalm accepts
both spellings) and must retype the receiver the same way, including
preserving a method-level template written as `self<U>`.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T
 */
final class Box
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @template U
     * @param U $value
     * @psalm-this-out self<U>
     */
    public function replace($value): void
    {
    }
}

function test(): void {
    $box = new Box(1);
    $box->replace("hello");
    /** @mir-check $box is Box<string> */
    $_ = 1;
}
===expect===
