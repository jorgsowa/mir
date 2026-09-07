===description===
`get_debug_type($obj->prop) === Foo::class` (both operand orders) and
`=== 'Foo'` all proved the property non-`Foo`-or-not but never called
narrow_receiver_non_null_on_prop_match, unlike the identical get_class()
idiom right next to them in the dispatch — $obj itself stayed nullable.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullPropertyFetch
===file===
<?php
class Foo {}

final class Holder {
    /** @var Foo|null */
    public $obj;
}

function narrowsClassConst(?Holder $h): void {
    if (get_debug_type($h->obj) === Foo::class) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function narrowsClassConstSymmetric(?Holder $h): void {
    if (Foo::class === get_debug_type($h->obj)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function narrowsStringLiteral(?Holder $h): void {
    if (get_debug_type($h->obj) === 'Foo') {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}
===expect===
