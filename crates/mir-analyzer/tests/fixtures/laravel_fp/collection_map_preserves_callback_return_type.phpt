===description===
Regression (mir-labs): `Post::all()` builds its collection exclusively through
`Post::fromFile()`. Its `map()` result must therefore retain `Post` as the
collection value type, so `findBySlug()` returns `Post|null`, not `mixed`.
===config===
suppress=MissingReturnType,MissingParamType,MissingPropertyType,UnusedParam
===file===
<?php

/**
 * @template TKey of array-key
 * @template TValue
 */
class Collection {
    /**
     * @template TMapValue
     * @param callable(TValue, TKey): TMapValue $callback
     * @return Collection<TKey, TMapValue>
     */
    public function map(callable $callback): Collection { return new Collection(); }

    /** @return TValue|null */
    public function firstWhere(string $key, string $value) { return null; }
}

final class Post {
    public static function all(): Collection {
        /** @var Collection<int, string> $paths */
        $paths = new Collection();

        return $paths->map(fn (string $path) => self::fromFile($path));
    }

    public static function findBySlug(string $slug): ?self {
        return self::all()->firstWhere('slug', $slug);
    }

    private static function fromFile(string $path): self { return new self(); }
}

$post = Post::findBySlug('hello');
/** @mir-check $post is Post|null */
===expect===
