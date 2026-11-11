===description===
`@psalm-if-this-is Box<int>` is checked against the receiver that a namespaced class's `@psalm-this-out self<U>` left behind.
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
    public function set(mixed $value): void
    {
    }

    /** @psalm-if-this-is Box<int> */
    public function needsInt(): void
    {
    }
}

function example(): void
{
    $box = new Box(1);
    $box->needsInt();
    $box->set('x');
    $box->needsInt();
//  ^^^^^^^^^^^^^^^^ IfThisIsMismatch: Cannot call App\Box::needsInt() — @if-this-is requires $this to be 'App\Box<int>', but it is 'App\Box<string>'
}
