===description===
A declared method on a class that also has `@method` tags still rejects a float for `int` under strict_types.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @method static addWeeks(int $weeks = 1) */
class Date {
    public function __call(string $name, array $args): mixed { return $this; }
    public function addDays(int $days): static { return $this; }
}

function shift(Date $date, float $days): void {
    $date->addDays($days);
//                 ^^^^^ InvalidArgument: Argument $days of addDays() expects 'int', got 'float'
}
