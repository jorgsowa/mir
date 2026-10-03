===description===
class-string<Handler<R, Q>> accepts implementing classes, and a template only mentioned in another template's bound (Q of Query<R>) is inferred from Q's binding
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template R */
interface Query {}

/**
 * @template R
 * @template Q of Query<R>
 */
interface Handler {
    /** @param Q $q @return R */
    public function handle(Query $q): mixed;
}

/** @implements Query<string> */
final class GetName implements Query {}
/** @implements Handler<string, GetName> */
final class GetNameHandler implements Handler {
    public function handle(Query $q): mixed { return 'x'; }
}
final class Unrelated {}

final class Module {
    /**
     * @template R
     * @template Q of Query<R>
     * @param class-string<Q> $query
     * @param class-string<Handler<R, Q>> $handler
     * @return Handler<R, Q>
     */
    public function register(string $query, string $handler): Handler { return new $handler(); }

    /**
     * @template R
     * @template Q of Query<R>
     * @param class-string<Q> $query
     * @return R
     */
    public function resultOf(string $query): mixed { return null; }

    /**
     * @template R
     * @template Q of Query<R>
     * @param Q $query
     * @return R
     */
    public function direct(Query $query): mixed { return null; }

    public function boot(): void {
        $h = $this->register(GetName::class, GetNameHandler::class);
        /** @mir-check $h is Handler<string, GetName> */
        echo 1;
        $r = $this->resultOf(GetName::class);
        /** @mir-check $r is string */
        echo 1;
        $d = $this->direct(new GetName());
        /** @mir-check $d is string */
        echo 1;
        $this->register(GetName::class, Unrelated::class);
//                                      ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $handler of register() expects 'class-string<Handler>', got 'class-string<Unrelated>'
    }
}
===expect===
