===description===
A local `@var`/`@phpstan-var` annotation inside a method body referencing
the enclosing *class's* own `@template` (as opposed to a per-method one)
must also resolve to a `TTemplateParam`, not an ordinary (and namespace-
mis-qualified) class reference.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
namespace App;

/**
 * @template T
 */
class Box {
    /** @var array<string, T> */
    private $items = [];

    /** @param string $key */
    public function get($key) {
        /** @var T $item */
        $item = $this->items[$key];
        return $item;
    }
}
===expect===
