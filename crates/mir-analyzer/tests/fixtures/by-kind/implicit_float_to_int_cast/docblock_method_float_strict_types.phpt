===description===
`__call` takes its arguments as an array, so a float for a `@method` `int` parameter under strict_types is the non-strict cast, not an invalid argument.
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
}

function shift(Date $date, float $weeks): void {
    $date->addWeeks(round($weeks) - 0.5);
//                  ^^^^^^^^^^^^^^^^^^^ ImplicitFloatToIntCast: Implicit cast from float to int truncates the fractional part
}
