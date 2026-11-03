===description===
Inside a trait, a class-string narrowed to `self::class` names the using class, so a constant fetch through it is not a direct trait access.
===file===
<?php
trait LazyDecorator {
    public static function scopes(?object $instance = null): mixed {
        $class = $instance ? $instance::class : static::class;
        if (self::class === $class && \defined($class.'::SCOPES')) {
            return $class::SCOPES;
        }
        return [];
    }
}
