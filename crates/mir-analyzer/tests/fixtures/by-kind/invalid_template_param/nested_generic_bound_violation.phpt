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
//               ^^ UnusedParam: Parameter $c is never used

$c = new Collection(new Cat());
process($c);
//<^^^^^^^^^^^ InvalidArgument: Argument $c of process() expects 'Collection<Animal>', got 'Collection<Cat>'
