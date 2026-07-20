===description===
A trait's own `@template T` no longer collides with the using class's own,
same-letter `@template T` — `inherited_template_bindings` never walked
`class.traits()`, so the trait's T had no entry and silently fell through
to the receiver's own T binding when substituting the trait-owned
property's type.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T */
trait BoxTrait {
    /** @var T */
    public $traitValue;
}

/** @template T */
class Container {
    use BoxTrait;
    /** @var T */
    public $value;
}

function takesInt(int $x): void {}
function takesString(string $x): void {}

/** @param Container<int> $c */
function f(Container $c): void {
    takesInt($c->value);
    takesString($c->traitValue);
}
===expect===
MixedArgument@21:16-21:30: Argument $x of takesString() is mixed
