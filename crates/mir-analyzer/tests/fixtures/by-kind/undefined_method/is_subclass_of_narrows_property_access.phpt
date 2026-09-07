===description===
`is_subclass_of($obj->prop, X::class)` must narrow the property receiver like
the already-correct variable case.
===config===
suppress=MissingConstructor,PossiblyNullArgument,MissingPropertyType
===file===
<?php
class Animal {}
class Dog extends Animal {
    public function bark(): void {}
}
class Container {
    /** @var Animal|Dog|null */
    public $pet;
}
function f(Container $c): void {
    if (is_subclass_of($c->pet, 'Animal')) {
        $c->pet->bark();
    }
}
===expect===
