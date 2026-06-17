===description===
A `mixed | null` receiver yields only MixedMethodCall, never PossiblyNullMethodCall.
The union arises from a `@template TValue` accessor declared `@return TValue|null`
used unbound (TValue → mixed), mirroring Laravel's `Fluent::__get`.
===config===
suppress=UnusedParam,MissingPropertyType,MixedPropertyFetch
===file===
<?php
/**
 * @template TValue
 */
class Bag {
    /**
     * @return TValue|null
     */
    public function __get(string $key): mixed
    {
        return null;
    }
}

function run(Bag $bag): void
{
    $bag->column->doSomething();
}
===expect===
MixedMethodCall@17:4-17:31: Method doSomething() called on mixed type
