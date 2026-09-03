===description===
`match ($x::class) { Foo::class => ... }` didn't narrow `$x` per arm — the
match-subject/arm-type intersection only recognized a variable, property, or
static-property subject, and even if it recognized `$x::class` the
intersection would operate on the class-string domain (not the object `$x`
itself), so it could never narrow the receiver anyway. Covers a plain
variable, property, and static-property `::class` receiver; an arm listing
more than one class (union narrowing, since comma-separated arm conditions
are OR semantics); and a string-literal arm condition.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class ErrorA {
    public function __construct(public string $aOnlyField) {}
    public function commonMsg(): string { return 'a'; }
}
class ErrorB {
    public function __construct(public string $bOnlyField) {}
    public function commonMsg(): string { return 'b'; }
}
class ErrorC {
    public function differentMsg(): string { return 'c'; }
}

function describe(ErrorA|ErrorB $error): string {
    return match ($error::class) {
        ErrorA::class => $error->aOnlyField,
        ErrorB::class => $error->bOnlyField,
    };
}

function describeStringLiteral(ErrorA|ErrorB $error): string {
    return match ($error::class) {
        'ErrorA' => $error->aOnlyField,
        'ErrorB' => $error->bOnlyField,
    };
}

function describeUnionArm(ErrorA|ErrorB|ErrorC $error): string {
    return match ($error::class) {
        ErrorA::class, ErrorB::class => $error->commonMsg(),
        ErrorC::class => $error->differentMsg(),
    };
}

class Box {
    public ErrorA|ErrorB $error;
}
function describeProp(Box $box): string {
    return match ($box->error::class) {
        ErrorA::class => $box->error->aOnlyField,
        ErrorB::class => $box->error->bOnlyField,
    };
}

class Registry {
    public static ErrorA|ErrorB $current;
    public static function describe(): string {
        return match (self::$current::class) {
            ErrorA::class => self::$current->aOnlyField,
            ErrorB::class => self::$current->bOnlyField,
        };
    }
}
===expect===
