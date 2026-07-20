===description===
`get_parent_class($obj->prop) === 'X'` narrows the receiver non-null in
BOTH branches — get_parent_class(null) throws a TypeError, so reaching
either comparison result proves the receiver was non-null. The helper
wrongly gated this on the comparison's own truth value instead of always
applying it.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch
===file===
<?php
class Base {}
class Foo extends Base {}
class Container {
    public ?Foo $child = null;
}

function trueBranchNarrowsReceiver(?Container $c): void {
    if (get_parent_class($c->child) === 'Base') {
        /** @mir-check $c is Container */
        $_ = 1;
    }
}

function falseBranchAlsoNarrowsReceiver(?Container $c): void {
    if (get_parent_class($c->child) !== 'Base') {
        /** @mir-check $c is Container */
        $_ = 1;
    }
}
===expect===
PossiblyNullArgument@9:25-9:34: Argument $object_or_class of get_parent_class() might be null
PossiblyNullArgument@16:25-16:34: Argument $object_or_class of get_parent_class() might be null
