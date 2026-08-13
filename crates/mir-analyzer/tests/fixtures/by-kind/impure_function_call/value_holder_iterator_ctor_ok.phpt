===description===
Constructing a value-holder iterator inside an @immutable method must not be
flagged as ImpureFunctionCall: passing `$this` (an immutable readonly receiver)
into a constructor that only reads it — copying its array state into the new
iterator — cannot mutate anything.
===config===
suppress=MissingConstructor,UnusedClass,InvalidReturnType
===file===
<?php
/** @psalm-immutable */
final readonly class TestSuiteCollection {
    /** @var list<TestSuite> */
    private array $testSuites;

    public function __construct(TestSuite ...$testSuites) {
        $this->testSuites = $testSuites;
    }

    /** @return list<TestSuite> */
    public function asArray(): array {
        return $this->testSuites;
    }

    /** @return TestSuiteCollectionIterator<int, TestSuite> */
    public function getIterator() {
        return new TestSuiteCollectionIterator($this);
    }
}

final class TestSuite {}

/** @implements Iterator<int, TestSuite> */
final class TestSuiteCollectionIterator implements \Iterator {
    /** @var list<TestSuite> */
    private readonly array $testSuites;

    public function __construct(TestSuiteCollection $collection) {
        $this->testSuites = $collection->asArray();
    }

    #[\ReturnTypeWillChange]
    public function current() {}

    #[\ReturnTypeWillChange]
    public function next(): void {}

    #[\ReturnTypeWillChange]
    public function key() {}

    #[\ReturnTypeWillChange]
    public function valid(): bool {
        return true;
    }

    #[\ReturnTypeWillChange]
    public function rewind(): void {}
}
===expect===
