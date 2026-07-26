===description===
`$cls::$prop = x` (a variable class-string receiver, e.g. a
`class-string<Foo>`-typed parameter) silently bypassed the static-property
purity/readonly check entirely — `resolve_static_prop_target` only
matched a literal class-name `Identifier`, never a variable holding a
resolved class-string type.
===config===
suppress=UnusedParam
===file===
<?php
class Registry {
    public static int $count = 0;
}

/** @pure */
function bump(string $cls): void {
    /** @var class-string<Registry> $cls */
    $cls::$count = 5;
}
===expect===
ImpureStaticPropertyAssignment@9:4-9:20: Assigning to static property Registry::$count in a @pure function
