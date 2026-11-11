===description===
Inside a namespace, `@psalm-self-out self<U>` and `@psalm-this-out static<U>` retype the receiver with the method's own template, instead of naming a class `App\self`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @template T
 */
class Box
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
    public function replace(mixed $value): void
    {
    }

    /**
     * @template U
     * @param U $value
     * @psalm-this-out static<U>
     */
    public function swap(mixed $value): void
    {
    }
}

function test(): void {
    $box = new Box(1);
    $box->replace('hello');
    /** @mir-check $box is App\Box<string> */
    $_ = 1;

    $box->swap(1.5);
    /** @mir-check $box is App\Box<float> */
    $_ = 1;
}
