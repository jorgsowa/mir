===description===
`array_key_exists('Iface', class_implements($this->service))` true
branch narrows the property via the same relationship a direct
`$this->service instanceof Iface` proves, but never called
narrow_receiver_non_null_on_prop_match like the direct instanceof arm
does — the receiver itself stayed nullable.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyInvalidArgument,PossiblyNullArgument,PossiblyNullPropertyFetch
===file===
<?php
interface Loggable {}
class Service implements Loggable {}

final class Holder {
    /** @var Service|null */
    public $service;
}

function narrowsReceiver(?Holder $h): void {
    if (array_key_exists('Loggable', class_implements($h->service))) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}
===expect===
