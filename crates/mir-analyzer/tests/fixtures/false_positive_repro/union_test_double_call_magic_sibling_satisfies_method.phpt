===description===
A `Real|TestDouble` union (the standard mocking-library autocomplete idiom)
must not flag UndefinedMethod for a method the "real" atom lacks when the
sibling test-double atom has a catch-all `__call` — the property is always
actually a test double at runtime, never the real class.
===file===
<?php
class RealService {
    public function doWork(string $s): string {
        return $s;
    }
}
class TestDoubleWrapper {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
    public function reveal(): RealService {
        return new RealService();
    }
}
class WidgetTest {
    private RealService|TestDoubleWrapper $service;
    public function __construct() {
        $this->service = new TestDoubleWrapper();
    }
    public function test(): void {
        $this->service->reveal();
    }
}
===expect===
