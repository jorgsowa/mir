===description===
trait require implements violation
===file===
<?php
interface Countable {}

/**
 * @psalm-require-implements Countable
 */
trait HasCount {
    public function count(): int { return 0; }
}

class Collection implements Countable {
    use HasCount;
}

class Bag {
    use HasCount;
}
===expect===
InvalidTraitUse@16:8: Trait HasCount used incorrectly: Class Bag uses trait HasCount but does not implement Countable
