===description===
P25 (non-promotion form): the same bare-builtin-name resolution bug also hits a
plain (non-promoted) property type hint and a plain method param type hint, not
just constructor promotion — both go through the same `resolve_union`/
`resolve_type_name` path.
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Holder
{
    private Generator $g;

    public function __construct(Generator $g)
    {
        $this->g = $g;
    }

    public function set(Generator $g): void
    {
        $this->g = $g;
    }

    public function run(): string
    {
        return $this->g->build();
    }
}
===expect===
