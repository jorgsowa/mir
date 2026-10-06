===description===
Interface method docblock reaches an undocumented implementation
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Source {
    /**
     * @template U
     * @param callable(T): U $f
     * @return Source<U>
     */
    public function map(callable $f): Source;
}

/**
 * @template T
 * @implements Source<T>
 */
final class ListSource implements Source {
    /** @param T $v */
    public function __construct(private mixed $v) {}
    public function map(callable $f): Source { return new ListSource($f($this->v)); }
}

function sink(mixed $x): void {}

/** @param ListSource<int> $s */
function run(ListSource $s): void {
    $r = $s->map(fn(int $i): float => $i / 2);
    /** @mir-check $r is Source<float> */
    sink($r);
}
