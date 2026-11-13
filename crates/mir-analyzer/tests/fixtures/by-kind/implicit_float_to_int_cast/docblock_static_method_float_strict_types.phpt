===description===
A float for a static `@method` `int` parameter under strict_types goes through `__callStatic`, so it is the non-strict cast.
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

/** @method static self fromWeeks(int $weeks) */
class Period {
    public static function __callStatic(string $name, array $args): mixed { return new self(); }
}

function make(float $weeks): void {
    Period::fromWeeks($weeks);
//                    ^^^^^^ ImplicitFloatToIntCast: Implicit cast from float to int truncates the fractional part
}
