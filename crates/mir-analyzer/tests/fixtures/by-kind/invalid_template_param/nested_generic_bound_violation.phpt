===description===
A bound like `T of Collection<Animal>` is enforced when the inferred binding is itself parameterized
===file===
<?php
class Animal {}
class Cat {}

/** @template V */
class Collection {
    /** @param V $item */
    public function __construct(private $item) {}
}

/**
 * @template T of Collection<Animal>
 * @param T $c
 */
function process($c): void {}

$c = new Collection(new Cat());
process($c);
===expect===
MissingPropertyType@8:32-8:45: Property Collection::$item has no type annotation
UnusedParam@15:17-15:19: Parameter $c is never used
InvalidTemplateParam@18:0-18:11: Template type 'T' inferred as 'Collection<Cat>' does not satisfy bound 'Collection<Animal>'
