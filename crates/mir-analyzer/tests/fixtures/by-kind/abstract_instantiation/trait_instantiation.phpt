===description===
A trait cannot be instantiated; `new self()` and `new static()` inside a trait name the using class.
===file===
<?php
trait Loggable {
    public static function make(): static {
        return new static();
    }

    public function copy(): self {
        return new self();
    }
}

final class Logger {
    use Loggable;
}

var_dump(new Loggable());
//           ^^^^^^^^ TraitInstantiation: Cannot instantiate trait Loggable
