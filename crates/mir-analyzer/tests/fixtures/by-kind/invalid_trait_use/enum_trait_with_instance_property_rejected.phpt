===description===
An enum using a trait that declares a non-static property is a hard PHP
fatal (enums may use traits for methods, but cannot carry extra state
beyond their cases). Enums previously tracked zero `use` declarations at
all — collect_enum silently dropped EnumMemberKind::TraitUse, so neither
this check nor trait-method resolution through class_ancestors_by_fqcn ever
saw an enum's traits.
===config===
suppress=UnusedParam
===file===
<?php
trait HasCounter {
    public int $count = 0;
}

trait Describable {
    public function describe(): string {
        return "a case";
    }
}

enum Status {
    use HasCounter;
    case Active;
    case Inactive;
}

enum Kind {
    use Describable;
    case A;
    case B;
}

function f(Kind $k): string {
    return $k->describe();
}
===expect===
InvalidTraitUse@13:8-13:18: Trait HasCounter used incorrectly: Enum Status cannot use trait HasCounter: it declares a non-static property $count, and enums cannot carry state beyond their cases
