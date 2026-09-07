===description===
P25 follow-up: the same builtin-name-shadowing bug reproduces in a second, separate
resolver — `call/opaque_callback.rs`'s interprocedural `array_map`/`array_filter`
callback-return inference resolves a literal closure's *native* return-type hint via
`resolve_union_for_file`/`resolve_docblock_type_name` (the docblock-leniency path),
not the strict native-hint resolver the collector uses. When a function taking a bare
`callable` param is itself called elsewhere with a literal closure whose native return
type is a same-namespace class shadowing a builtin (`Generator`), and that function's
own `array_map($cb, ...)` result is used by a third caller passing an opaque `$cb`,
the borrowed return type came back as the bare builtin name instead of the local FQCN.
Fixed by adding `resolve_union_for_file_native` (mirrors the split already made in
`collector::resolution` for the primary property/param/return path) and using it for
the two closure/arrow-fn native-hint sites in `opaque_callback.rs`.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

function mapper(callable $cb, array $items)
{
    return array_map($cb, $items);
}

function caller1(): void
{
    mapper(function (): Generator {
        return new Generator();
    }, []);
}

function caller2(callable $cb, array $items): void
{
    $result = mapper($cb, $items);
    foreach ($result as $r) {
        echo $r->build();
    }
}
===expect===
