===description===
Trait method that adds optional params beyond the parent signature must not
report TooManyArguments when called with those extra args. PHP semantics:
trait methods fully override the inherited parent method.
===file===
<?php
class Base {
    public function process(string $input): string {
        return $input;
    }
}

trait Enriched {
    public function process(string $input, int $flags = 0, ?bool $strict = null): string {
        return $input . $flags . ($strict ? '!' : '');
    }
}

class Concrete extends Base {
    use Enriched;
}

$c = new Concrete();
// Three args are valid per the trait signature — must not raise TooManyArguments.
$c->process('hello', 42, true);
===expect===
