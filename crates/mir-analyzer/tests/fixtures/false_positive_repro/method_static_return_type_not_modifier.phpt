===description===
FP-H: `@method static name()` — `static` is the PHP return type for fluent APIs, not
the static method modifier. When a subclass provides a concrete non-static `addDay()`
the parent docblock should not cause MethodSignatureMismatch.
===config===
php_version=8.2
===file===
<?php

/**
 * Carbon-style fluent docblock: `static` is the return type, not the modifier.
 *
 * @method static addDay(int $value = 1)
 * @method static subDay(int $value = 1)
 */
class Carbon {
    /** @return static */
    public function __call(string $name, array $args): static
    {
        return $this;
    }
}

class CarbonImmutable extends Carbon {
    /** @suppress UnusedParam */
    public function addDay(int $value = 1): static
    {
        return clone $this;
    }
}
===expect===
