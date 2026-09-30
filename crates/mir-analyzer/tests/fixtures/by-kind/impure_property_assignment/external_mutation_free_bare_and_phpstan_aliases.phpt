===description===
@external-mutation-free (bare) and @phpstan-external-mutation-free had no
recognized form at all -- only the bare "psalm-external-mutation-free"
string was matched, unlike every sibling purity tag (@pure,
@mutation-free, @immutable, @readonly), which all accept bare/psalm-/
phpstan- forms.
===file===
<?php
class Config {
    public string $mode = 'default';
}

class BareConfigurator {
    /** @external-mutation-free */
    public function configure(Config $cfg): void {
        $cfg->mode = 'active';
//      ^^^^^^^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property mode of a parameter in a pure or external-mutation-free context
    }
}

class PhpstanConfigurator {
    /** @phpstan-external-mutation-free */
    public function configure(Config $cfg): void {
        $cfg->mode = 'active';
//      ^^^^^^^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property mode of a parameter in a pure or external-mutation-free context
    }
}
===expect===
