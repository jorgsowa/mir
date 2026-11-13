===description===
A string that can never be an `int` stays an invalid argument for a `@method` parameter.
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

function shift(Date $date): void {
    $date->addWeeks('many');
//                  ^^^^^^ InvalidArgument: Argument $weeks of addWeeks() expects 'int', got '"many"'
}
