===description===
A local `@var`/`@phpstan-var` annotation inside a method body referencing
the enclosing method's own `@template` (not a class-level one) was resolved
through the template-unaware `resolve_union_for_file`, so the bare `T`
was namespace-qualified like an ordinary class name and then failed to
resolve — a false `UndefinedDocblockClass`. Modeled on doctrine/instantiator's
`Instantiator::get()`, which stores heterogeneous cached values in a
`static array` and casts back to `T` on read via `@phpstan-var`.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
namespace App;

class Instantiator {
    /** @var array<string, object> */
    private static $cache = [];

    /**
     * @template T of object
     * @param class-string<T> $className
     * @return T
     */
    public function get(string $className) {
        /** @phpstan-var T $cached */
        $cached = self::$cache[$className];
        return $cached;
    }
}
===expect===
