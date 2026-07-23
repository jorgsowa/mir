===description===
@psalm-self-out through a NULLSAFE property-access receiver
($h?->factory?->prepare()) still silently no-oped after
property_receiver_retypes_property.phpt's fix -- that fix only matched
extract_prop_access (plain `->`), not extract_any_prop_access, so the
nullsafe receiver form fell through unretyped.
===config===
suppress=UnusedParam,MissingConstructor,PossiblyNullMethodCall
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
    public ?Container $factory = null;
}

function test(?Holder $h): void {
    $h?->factory?->prepare();
    // Nullsafe short-circuit means the retyped result still carries |null
    // (matching the existing bare-variable-receiver nullsafe behavior) --
    // the bug this fixture targets is whether the retype to ReadyFactory
    // happens AT ALL for a nullsafe property-access receiver, not the null.
    /** @mir-check $h->factory is ReadyFactory|null */
    $_ = 1;
}
===expect===
