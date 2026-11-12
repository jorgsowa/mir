===description===
A property `@var` that the native type rules out (an array or object on a scalar, a class that isn't a subtype of the native class) is ignored in favor of the native type; a `@var` subclass still refines it.
===file===
<?php
interface Store { public function get(string $key): mixed; }
interface Factory { public function get(string $collection): Store; }
class ExpirableFactory implements Factory {
    public function get(string $collection): Store { throw new \RuntimeException($collection); }
}
final class ExpirableStore implements Store {
    public function get(string $key): mixed { return $key; }
    public function expire(): void {}
}

final class Fetcher {
    /** @var ExpirableFactory */
    protected Store $store;

    /** @var array */
    public string $classOrService;

    /** @var ExpirableStore */
    public Store $refined;

    public function __construct(Factory $factory, ?string $class = null) {
        $this->store = $factory->get('feed');
        $this->classOrService = $class ?? '';
        $this->refined = new ExpirableStore();
    }

    public function fetch(): bool {
        $this->refined->expire();
        return $this->store->get('items') === null;
    }
}

function take(string $s): string { return $s; }

function pass(Fetcher $f): void {
    take($f->classOrService);
}
