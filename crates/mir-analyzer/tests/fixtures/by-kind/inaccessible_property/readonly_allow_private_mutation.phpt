===description===
`@psalm-readonly-allow-private-mutation`, `@phpstan-readonly-allow-private-mutation` and `@readonly` with `@psalm-allow-private-mutation` make a property writable only inside its class, like `private(set)`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Counter
{
    /** @phpstan-readonly-allow-private-mutation */
    public int $value = 0;

    /** @psalm-readonly-allow-private-mutation */
    public int $total = 0;

    /**
     * @readonly
     * @psalm-allow-private-mutation
     */
    public int $last = 0;

    public function increment(): void
    {
        $this->value++;
        $this->total += 1;
        $this->last = 1;
    }
}

function bump(Counter $counter): int
{
    $counter->value = 99;
//  ^^^^^^^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property Counter::$value
    $counter->total = 99;
//  ^^^^^^^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property Counter::$total
    $counter->last = 99;
//  ^^^^^^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property Counter::$last
    return $counter->value + $counter->total + $counter->last;
}
