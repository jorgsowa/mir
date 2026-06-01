===description===
@method entries whose description text contains parentheses must not produce
phantom parameters. Carbon documents zero-arg magic methods like:
  @method $this addDay() Add one day to the instance (using date interval).
The old rfind(')') captured the description's closing paren, creating a
spurious required parameter and a TooFewArguments false positive.
===file===
<?php
/**
 * @method $this addDay() Add one day to the instance (using date interval).
 * @method $this subDay() Sub one day from the instance (using date interval).
 * @method $this addSecond() Add one second to the instance (using date interval).
 * @method $this addDays(int|float $value = 1) Add days (the $value count passed in).
 */
class DateHelper {
    public function __call(string $method, array $args): static {
        return $this;
    }
}

$d = new DateHelper();
// These must not raise TooFewArguments — the @method signatures have zero params.
$d->addDay();
$d->subDay();
$d->addSecond();
// Optional param — zero args still valid.
$d->addDays();
===expect===
