===description===
P25 regression guard: removing the builtin-name shortcut from native type-hint
resolution must not break the genuine global `Generator` when it's referenced via
an explicit `use Generator;` import and there's no local class of that name — the
`use`-alias lookup in `resolve_type_name` still runs before namespace-qualification
either way, so this path was never the buggy one.
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

use Generator;

final class Producer
{
    public function make(): Generator
    {
        return (function (): Generator {
            yield 1;
        })();
    }

    public function run(): bool
    {
        return $this->make()->valid();
    }
}
===expect===
