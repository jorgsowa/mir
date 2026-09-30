===description===
Assigning to a property of a parameter fires ImpurePropertyAssignment inside a
@psalm-external-mutation-free method.
===file===
<?php

class Config {
    public string $mode = 'default';
}

class Configurator {
    /** @psalm-external-mutation-free */
    public function configure(Config $cfg): void {
        $cfg->mode = 'active';
//      ^^^^^^^^^^^^^^^^^^^^^ ImpurePropertyAssignment: Assigning to property mode of a parameter in a pure or external-mutation-free context
    }
}
===expect===
