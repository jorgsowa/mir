===description===
FP: `new $var(...)` where `$var` holds an object instance is valid PHP —
constructs a fresh instance of that object's own runtime class. Common
"rethrow with a richer exception" idiom.
===config===
suppress=MissingReturnType
===file===
<?php
function enrich(\Throwable $e): string {
    return 'context: ' . $e->getMessage();
}

function risky(): void {
    throw new \RuntimeException('boom');
}

function wrap(): void {
    try {
        risky();
    } catch (\Throwable $e) {
        throw new $e(enrich($e), $e->getCode(), $e);
    }
}
===expect===
