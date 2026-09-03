===description===
FP-C6: `ast\Metadata` plus the remaining introspection functions
(get_metadata/get_supported_versions/kind_uses_flags) must resolve too —
covers every symbol in the ast/ast.php stub, not just the parse entry points.
===config===
suppress=UnusedParam
===file===
<?php

function describeAll(): void {
    foreach (ast\get_metadata() as $meta) {
        echo $meta->name;
        echo $meta->kind;
        echo $meta->flagsCombinable;
    }
    foreach (ast\get_supported_versions(true) as $version) {
        echo $version;
    }
}

function supportsFlags(int $kind): bool {
    return ast\kind_uses_flags($kind);
}
===expect===
