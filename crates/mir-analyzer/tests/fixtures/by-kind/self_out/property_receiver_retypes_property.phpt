===description===
@psalm-self-out on a call through a property-access receiver (`$obj->prop
->method()`) previously silently no-oped — only a bare variable receiver
was retyped. Retype the property via set_prop_refined instead, mirroring
how a variable receiver is retyped, so a subsequent access no longer
false-positives as calling an undefined method on the property's original
(declared) type.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Factory {}
class ReadyFactory extends Factory {
    public function build(): string {
        return "built";
    }
}

class Container {
    /** @psalm-self-out ReadyFactory */
    public function prepare(): void {}
}

class Holder {
    public Container $factory;
}

function test(Holder $h): void {
    $h->factory->prepare();
    /** @mir-check $h->factory is ReadyFactory */
    $_ = 1;
    $h->factory->build();
}
===expect===
