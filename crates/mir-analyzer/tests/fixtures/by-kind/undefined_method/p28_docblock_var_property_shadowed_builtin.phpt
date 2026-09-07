===description===
P28 sibling: a `@var` property docblock naming a same-namespace class that
shadows a builtin (`Generator`) must resolve to that local class, matching
its already-correct native type hint, instead of carrying the builtin's
bare-name leniency into every reader of the property's stored type
(`find_property_in_chain`).
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Holder
{
    /** @var Generator */
    private Generator $g;

    public function __construct(Generator $g)
    {
        $this->g = $g;
    }

    public function run(): string
    {
        return $this->g->build();
    }
}
===expect===
