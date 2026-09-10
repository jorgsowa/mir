===description===
M19: a closure/arrow returning a provably non-empty `.` concat kept its
declared `string` return, so `array_map` produced `list<string>` and was
flagged InvalidArgument for a `list<non-empty-string>` parameter — the
shape of phpunit's TestUI/TestSuiteFilterProcessor.php. The recorded
closure/arrow return refines to non-empty-string when the declared type
and the body are both string-family and the body is provably non-empty.
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
