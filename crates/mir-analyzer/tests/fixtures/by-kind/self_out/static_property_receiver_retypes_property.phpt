===description===
@psalm-self-out through a STATIC-property-access receiver
(Holder::$factory->prepare()) silently no-oped -- the write-back only
matched a bare variable or an (any-form) instance-property receiver,
never extract_static_prop_access.
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
    public static Container $factory;
}

function test(): void {
    Holder::$factory->prepare();
    /** @mir-check Holder::$factory is ReadyFactory */
    $_ = 1;
    Holder::$factory->build();
}
===expect===
