===description===
P25 follow-up: a closure/arrow-function literal's own native return-type hint is
resolved via `resolve_named_objects_in_union` (`expr/helpers.rs`), used directly by
`array_map`'s callback-return inference (`Atomic::TClosure` carries its return type
straight from this resolution) — this shares the exact docblock-leniency bug with the
primary collector path P25 fixed, just for closure/arrow-fn literals specifically. An
inline closure passed directly to `array_map`, returning a same-namespace class that
shadows a builtin iterator name, had its return type resolved to the bare builtin
instead of the local FQCN, so every member access on the mapped result flagged
UndefinedMethod. Fixed by adding `resolve_named_objects_in_union_native` and using it
for closure/arrow-fn return-type hints (`expr/closures.rs`) and closure/arrow-fn
param-type hints (`expr/helpers.rs::ast_params_to_fn_params_resolved`).
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

function useDirect(array $items): void
{
    $result = array_map(function (): Generator {
        return new Generator();
    }, $items);
    foreach ($result as $r) {
        echo $r->build();
    }
}
===expect===
