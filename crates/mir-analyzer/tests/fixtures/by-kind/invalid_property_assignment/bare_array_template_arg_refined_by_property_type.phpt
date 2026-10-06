===description===
A bare `array` inferred for an invariant template argument carries no key/value info, so a property typed with a more specific array argument accepts it; unrelated arguments are still rejected.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template A */
final class Lazy {
    /** @param Closure(): A $make */
    public function __construct(private Closure $make) {}
}

final class Holder {
    /** @var Lazy<array<non-empty-string, mixed>> */
    public Lazy $keyed;
    /** @var Lazy<list<int>> */
    public Lazy $list;
    /** @var Lazy<int> */
    public Lazy $scalar;
    /** @var Lazy<array{id: int}> */
    public Lazy $shape;

    public function __construct() {
        $this->keyed = new Lazy(fn(): array => array_merge([], []));
        $this->list = new Lazy(fn(): array => []);
        $this->shape = new Lazy(fn(): array => ['id' => 1]);
        $this->scalar = new Lazy(fn(): array => []);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $scalar expects 'Lazy<int>', cannot assign 'Lazy<array>'
    }
}
===expect===
