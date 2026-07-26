===description===
@psalm-external-mutation-free / @psalm-mutation-free on a FREE function
(not a method) never checked property writes on its parameters at all —
FunctionDef had no is_mutation_free/is_external_mutation_free fields, so
the tags were parsed and silently discarded. A free function has no
$this, so @mutation-free is behaviorally the same as
@external-mutation-free here: both forbid mutating a parameter's
property.
===file===
<?php
class Config {
    public string $mode = 'default';
}

/** @psalm-external-mutation-free */
function configureExternal(Config $cfg): void {
    $cfg->mode = 'active';
}

/** @psalm-mutation-free */
function configureMutationFree(Config $cfg): void {
    $cfg->mode = 'active';
}
===expect===
ImpurePropertyAssignment@8:4-8:25: Assigning to property mode of a parameter in a pure or external-mutation-free context
ImpurePropertyAssignment@13:4-13:25: Assigning to property mode of a parameter in a pure or external-mutation-free context
