===description===
An inline `@var class-string<T>` names the enclosing function's template, not a class, so passing it to a templated callee satisfies that callee's bound.
===file===
<?php
interface Shape {}
interface Named {}

interface Registry
{
    /**
     * @template T of Shape&Named
     * @param string|class-string<T> $name
     * @return Closure():T
     */
    public function get(string $name): Closure;
}

final class Loader
{
    /** @template T of Named&Shape */
    public static function load(Registry $registry, string $name): void
    {
        /** @var class-string<T> $name */
        $registry->get($name);
    }
}

/** @template T of Named&Shape */
function loadFree(Registry $registry, string $name): void
{
    /** @var class-string<T> $name */
    $registry->get($name);
}
