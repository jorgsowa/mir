===description===
P23 (property-fetch form): Psalm's `PossiblyNullReference` covers member access on a
possibly-null receiver regardless of whether it's a method call or a property fetch;
mir splits that into two separate kinds (`PossiblyNullMethodCall`/`PossiblyNullPropertyFetch`),
so the alias must map to both — this pins the property-fetch half.
===file===
<?php
class Foo {
    public int $prop = 1;
}
function test(?Foo $obj): void {
    /** @psalm-suppress PossiblyNullReference */
    echo $obj->prop;
}
===expect===
