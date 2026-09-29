===description===
A parent docblock param typed as an intersection (`array<string,mixed>&array{..}`) is a subtype
of bare `array`, so an override widening to `array`/`iterable`/`array|null` is valid contravariance.
Narrowing to an unrelated type is still flagged.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type Context = array<string, mixed> & array{actor: array{id: int|string} & array<string, mixed>}
 */
interface Logger {
    /** @psalm-param Context $context */
    public function info(array $context): void;

    /** @psalm-param Context $context */
    public function warn(array $context): void;

    /** @psalm-param Context $context */
    public function err(array $context): void;

    /** @psalm-param Context $context */
    public function bad(array $context): void;
}

final class Impl implements Logger {
    public function info(array $context): void {
        /** @mir-check $context is array */
    }
    public function warn(iterable $context): void {}
    public function err(?array $context): void {}
    public function bad(string $context): void {}
}
===expect===
MethodSignatureMismatch@25:4-25:49: Method Impl::bad() signature mismatch: parameter $context type 'string' is narrower than parent type 'array<string, mixed>&array{'actor': array{'id': int|string}&array<string, mixed>}'
