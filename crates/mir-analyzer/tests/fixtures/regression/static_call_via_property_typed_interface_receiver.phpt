===description===
A static call through a property-access receiver typed as an interface
($h->provider::method()) is the same dynamic-dispatch shape as a plain
variable receiver — must analyze clean.
===config===
suppress=MissingConstructor
===file===
<?php

interface Provider {
    public static function getDefinitions(): array;
}
class Holder {
    public Provider $provider;
}

function run(Holder $h): void {
    $h->provider::getDefinitions();
}
===expect===
