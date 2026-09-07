===description===
`if ($obj->prop = expr)` narrowed the property's truthy/falsy type but
never narrowed the receiver non-null, even though reaching the
assignment at all — regardless of which branch is taken — already
proves it: PHP fatals assigning a property on a null receiver, unlike a
plain read.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var string */
    public $name;
}

function narrowsReceiverTrueBranch(?Holder $h): void {
    if ($h->name = computeName()) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function narrowsReceiverFalseBranch(?Holder $h): void {
    if ($h->name = computeName()) {
    } else {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function computeName(): string {
    return 'x';
}
===expect===
