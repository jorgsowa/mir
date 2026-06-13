===description===
No ImplicitToStringCast when arg type directly satisfies a non-string arm of the union
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \Throwable|string $exception
 */
function reportLike($exception): void {}

try {
    throw new \RuntimeException("error");
} catch (\Throwable $e) {
    reportLike($e);
}

class HasToString {
    public function __toString(): string { return 'x'; }
}
/**
 * @param string|int $val
 */
function takesStringOrInt($val): void {}
takesStringOrInt(new HasToString());
===expect===
ImplicitToStringCast@20:18-20:35: Class HasToString is implicitly cast to string
