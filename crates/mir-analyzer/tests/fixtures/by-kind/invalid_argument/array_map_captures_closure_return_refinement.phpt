===description===
`array_map()` preserves a callback's inferred non-empty-string return type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/**
 * @param list<non-empty-string> $out
 */
function sink(array $out): void {
    $_ = $out;
}

/** @param list<string> $names */
function fromArrow(array $names): void {
    $mapped = array_map(static fn (string $name): string => 'prefix_' . $name, $names);
    /** @mir-check $mapped is list<non-empty-string> */
    sink($mapped);
}

/** @param list<string> $names */
function fromClosure(array $names): void {
    $mapped = array_map(
        static function (string $name): string {
            return 'prefix_' . $name;
        },
        $names,
    );
    /** @mir-check $mapped is list<non-empty-string> */
    sink($mapped);
}

/** @param list<string> $names */
function nullable(array $names): void {
    $mapped = array_map(
        static fn (string $name): ?string => $name !== '' ? $name : null,
        $names,
    );
    /** @mir-check $mapped is list<non-empty-string|null> */
    $_ = $mapped;
}

/** @param list<string> $names */
function staysString(array $names): void {
    $mapped = array_map(static fn (string $name): string => $name, $names);
    /** @mir-check $mapped is list<string> */
    $_ = $mapped;
}
===expect===
