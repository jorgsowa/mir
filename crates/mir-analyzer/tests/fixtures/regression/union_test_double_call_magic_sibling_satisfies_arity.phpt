===description===
A `Real|TestDouble` union (the standard mocking-library autocomplete idiom)
must not flag TooFewArguments/TooManyArguments on the "real" atom's own
arity when a sibling test-double atom has a catch-all `__call` — the real
atom's signature can never actually be enforced at runtime if the value is
in fact the test double, which accepts any arity via `__call`.
===file===
<?php
class RealService {
    public function doSomething(int $a, int $b): void {
        echo $a + $b;
    }
}
class MagicProxy {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
}
class T {
    private MagicProxy|RealService $service;
    public function __construct() {
        $this->service = new MagicProxy();
    }
    public function tooFew(): void {
        $this->service->doSomething(1);
    }
    public function tooMany(): void {
        $this->service->doSomething(1, 2, 3);
    }
}
===expect===
