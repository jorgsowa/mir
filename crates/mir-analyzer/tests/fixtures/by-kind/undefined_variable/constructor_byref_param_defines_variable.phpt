===description===
Passing an undefined variable to a constructor's by-ref param defines it with
the param's type, or its `@param-out` type; a by-value param still doesn't.
===file===
<?php
final class Reader {
    /** @param-out list<string> $warnings */
    public function __construct(string $path, ?string &$error, ?array &$warnings = null) {
        $error = $path === '' ? 'empty' : null;
        $warnings = [];
    }
}

final class Sink {
    public function __construct(public mixed $value) {}
}

function open(string $path): Reader {
    $reader = new Reader($path, $error, warnings: $warnings);
    /** @mir-check $error is string|null */
    /** @mir-check $warnings is list<string> */
    echo $error, count($warnings);
    return $reader;
}

function overwrite(string $path): void {
    $error = 'unset';
    $warnings = null;
    new Reader($path, $error, $warnings);
    /** @mir-check $error is string|null */
    /** @mir-check $warnings is list<string> */
    echo $error, count($warnings);
}

function byValue(): Sink {
    return new Sink($value);
//                  ^^^^^^ UndefinedVariable: Variable $value is not defined
}
