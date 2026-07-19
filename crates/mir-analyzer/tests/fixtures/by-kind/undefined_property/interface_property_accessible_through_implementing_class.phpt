===description===
`@property-read` on an interface is a valid access target through a class
that `implements` it, not just when the variable is typed as the interface
directly — `ClassLike::own_properties()` dropped `InterfaceDef.own_properties`
entirely, so the ancestor-chain lookup used for the class case never saw it.
===config===
suppress=MissingConstructor,MixedReturnStatement
===file===
<?php
/** @property-read int $count */
interface Countable2 {}

class Impl implements Countable2 {
}

function throughInterface(Countable2 $c): int {
    return $c->count;
}

function throughImplementingClass(Impl $i): int {
    return $i->count;
}

function stillFlagsRealUndefinedProperty(Impl $i): int {
    return $i->nope;
}
===expect===
UndefinedProperty@17:15-17:19: Property Impl::$nope does not exist
