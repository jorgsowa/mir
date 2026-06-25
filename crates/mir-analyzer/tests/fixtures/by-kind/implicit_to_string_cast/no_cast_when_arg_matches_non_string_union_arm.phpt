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
===expect===
