===description===
`@return static<TKey, TMapValue>` returns the receiver class with those args, not the receiver's own.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnnecessaryVarAnnotation errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @template TKey of array-key
 * @template TValue
 */
class Collection {
    /** @param array<TKey, TValue> $items */
    public function __construct(public array $items = []) {}

    /**
     * @template TMapValue
     * @param callable(TValue, TKey): TMapValue $callback
     * @return static<TKey, TMapValue>
     */
    public function map(callable $callback) { return new static(); }

    /**
     * @param callable(TValue): bool $callback
     * @return static
     */
    public function filter(callable $callback) { return $this; }

    /** @return TValue|null */
    public function first() { return null; }

    /** @return static<TValue, TKey> */
    public function flip() { return new static(); }

    /** @return static<TKey, TValue>|Collection<TKey, TValue> */
    public function keepOrBase() { return $this; }

    /** @return ($preserveKeys is true ? static<int, static> : static<int, static<int, TValue>>) */
    public function chunk(int $size, bool $preserveKeys = false) { return new static(); }

    /**
     * @template TNewKey of array-key
     * @param (callable(TValue, TKey): TNewKey)|string $keyBy
     * @return static<($keyBy is string ? array-key : TNewKey), TValue>
     */
    public function keyBy($keyBy) { return new static(); }
}

/**
 * @template TKey of array-key
 * @template TModel
 * @extends Collection<TKey, TModel>
 */
class ModelCollection extends Collection {}

/** @extends Collection<int, string> */
class Lines extends Collection {
    /** @return static<int, string, bool> */
    public function odd() { return $this; }
}

final class Post {
    public function __construct(public string $title = '') {}
}

/** @var Collection<int, string> $c */
$c = new Collection(['a', 'b']);
$posts = $c->map(fn (string $s) => new Post($s));
/** @mir-check $posts is Collection<int, Post> */
$first = $posts->filter(fn (Post $p) => $p->title !== '')->first();
/** @mir-check $first is Post|null */
$_ = 1;

/** @var ModelCollection<int, string> $m */
$m = new ModelCollection();
$mapped = $m->map(fn (string $s) => strlen($s));
/** @mir-check $mapped is ModelCollection<int, int<0, max>> */
$_ = 1;

$kept = $posts->keepOrBase();
/** @mir-check $kept is Collection<int, Post> */
$_ = 1;

$lines = (new Lines())->odd();
/** @mir-check $lines is Lines<int, string> */
$_ = 1;

$chunks = $posts->chunk(2);
/** @mir-check $chunks is Collection<int, Collection<int, Post>> */
$_ = 1;

$byTitle = $posts->keyBy(fn (Post $p): int => strlen($p->title));
/** @mir-check $byTitle is Collection<int<0, max>, Post> */
$_ = 1;

/** @var Collection<int, string|null> $names */
$names = new Collection();
$flipped = $names->flip();
/** @mir-check $flipped is Collection<string, int> */
$_ = 1;

/** @return Collection<int, Post> */
function rawReceiverStaysRaw(Collection $raw): Collection {
    return $raw->map(fn ($item) => $item);
}
