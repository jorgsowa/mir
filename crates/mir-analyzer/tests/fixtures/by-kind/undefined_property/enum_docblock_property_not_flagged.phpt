===description===
`@property-read` docblock tags on an enum are parsed into `dummy_properties`
then discarded — `EnumDef` had no `own_properties` field at all, unlike
Class/Trait/Interface, so a documented virtual property always fired
UndefinedProperty. A real undefined property still gets flagged.
===config===
suppress=UnusedParam,MixedReturnStatement
===file===
<?php
/** @property-read string $label */
enum Status {
    case Active;
    case Inactive;

    public function __get(string $name): string {
        return $name === 'label' ? 'active-label' : '';
    }
}

function test(Status $s): string {
    return $s->label;
}

function stillFlagsRealUndefinedProperty(Status $s): string {
    return $s->nope;
}
===expect===
UndefinedProperty@17:15-17:19: Property Status::$nope does not exist
