===description===
A `mixed | null` argument is treated as `mixed`, not possibly-null: it yields
MixedArgument, never PossiblyNullArgument. The union arises from a `@template
TValue` magic accessor declared `@return TValue|null` used unbound (TValue →
mixed), so substitution produces an un-normalized `mixed | null`. Mirrors
Laravel's `Fluent::__get` (`@return TValue|null`) on a bare `Fluent` command.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
/**
 * @template TValue
 */
class Bag {
    /**
     * @return TValue|null
     */
    public function __get(string $key)
    {
        return null;
    }
}

function takesString(string $s): void {}

function run(Bag $bag): void
{
    takesString($bag->column);
}
===expect===
MixedArgument@19:16-19:28: Argument $s of takesString() is mixed
