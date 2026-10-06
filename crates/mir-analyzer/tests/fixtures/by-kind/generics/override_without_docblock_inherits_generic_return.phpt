===description===
Override without docblock inherits the parent's generic @return and callable param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template-covariant T
 * @template-covariant E
 */
abstract class Result {
    /**
     * @template U
     * @param Closure(T): U $f
     * @return Result<U, E>
     */
    abstract public function map(Closure $f): Result;
}

/**
 * @template-covariant V
 * @extends Result<V, never>
 */
final class Ok extends Result {
    /** @param V $v */
    public function __construct(public mixed $v) {}
    public function map(Closure $f): Result { return new Ok($f($this->v)); }
}

/**
 * @template-covariant X
 * @extends Result<never, X>
 */
final class Err extends Result {
    /** @param X $e */
    public function __construct(public mixed $e) {}
    public function map(Closure $f): Result { return $this; }
}

final class Catalog {}
final class Failure {}
final class Row {}

/** @template Y */
final class Page {
    /** @param list<Y> $items */
    public function __construct(public array $items) {}
}

/** @return Ok<Catalog> */
function okCatalog(): Ok { return new Ok(new Catalog()); }
/** @return Err<Failure> */
function errFailure(): Err { return new Err(new Failure()); }
/** @return Result<Catalog, Failure> */
function anyResult(): Result { return new Ok(new Catalog()); }

function sink(mixed ...$xs): void {}

/** @param Closure(Catalog): Page<Row> $getPage */
function run(Closure $getPage): void {
    $ok = okCatalog()->map($getPage);
    /** @mir-check $ok is Result<Page<Row>, never> */
    $err = errFailure()->map($getPage);
    /** @mir-check $err is Result<Page<Row>, Failure> */
    $base = anyResult()->map($getPage);
    /** @mir-check $base is Result<Page<Row>, Failure> */
    $inline = okCatalog()->map(fn(Catalog $c) => new Page([new Row()]));
    /** @mir-check $inline is Result<Page<Row>, never> */
    sink($ok, $err, $base, $inline);
}
