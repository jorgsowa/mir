===description===
class-string<Handler<R, Q>> accepts an implementing class and still rejects an unrelated one
===config===
suppress=UnusedParam
===file===
<?php
/** @template R */
interface Query {}

/**
 * @template R
 * @template Q of Query<R>
 */
interface Handler {}

final class GetName implements Query {}
/** @implements Handler<string, GetName> */
final class GetNameHandler implements Handler {}
final class Unrelated {}

final class Module {
    /**
     * @template R
     * @template Q of Query<R>
     * @param class-string<Q> $query
     * @param class-string<Handler<R, Q>> $handler
     */
    public function register(string $query, string $handler): void {}

    public function boot(): void {
        $this->register(GetName::class, GetNameHandler::class);
        $this->register(GetName::class, Unrelated::class);
    }
}
===expect===
InvalidArgument@27:40-27:56: Argument $handler of register() expects 'class-string<Handler>', got 'class-string<Unrelated>'
