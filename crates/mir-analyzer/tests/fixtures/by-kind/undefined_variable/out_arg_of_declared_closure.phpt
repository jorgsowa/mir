===description===
A callee declared only as `Closure` may take any argument by reference, so an undefined variable passed to it may be an out-param.
===file===
<?php
final class Pool {
    private static ?\Closure $merge = null;

    public function __construct() {
        self::$merge ??= \Closure::bind(
            static function (array $deferred, &$expiredIds): array {
                $expiredIds = [];
                return $deferred;
            },
            null,
            self::class
        );
    }

    public function commit(): bool {
        (self::$merge)([], $expiredIds);
        return $expiredIds !== [];
    }
}
