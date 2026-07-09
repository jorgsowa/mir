===description===
A `readonly class` requires every property it carries — including ones
pulled in from a used trait — to itself be readonly. check_trait_constraints
previously only cross-checked a trait's require-extends/require-implements
constraints; it never looked at the class-level `readonly` modifier against
properties contributed by a trait.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
trait HasName {
    public string $name = "x";
}
readonly class Person {
    use HasName;
}

trait HasReadonlyId {
    public readonly int $id;
}
readonly class Account {
    use HasReadonlyId;
    public function __construct(int $id) {
        $this->id = $id;
    }
}
===expect===
InvalidTraitUse@6:8-6:15: Trait HasName used incorrectly: Readonly class Person cannot use trait HasName: it declares a non-readonly property $name
