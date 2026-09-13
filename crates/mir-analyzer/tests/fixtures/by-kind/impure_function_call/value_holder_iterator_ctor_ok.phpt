===description===
Immutable methods can construct iterators from immutable receivers.
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
