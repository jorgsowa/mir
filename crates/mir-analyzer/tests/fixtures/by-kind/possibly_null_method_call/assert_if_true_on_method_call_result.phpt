===description===
`@psalm-assert-if-true \Exception $this->getException()` narrows the receiver's own
zero-arg call result in the caller when repeated calls are provably equal (a pure or
mutation-free method, or a method of an immutable class); otherwise, or after a mutating
call on the receiver, the call result is not narrowed.
===file===
<?php
/** @psalm-immutable */
final class Result {
    public function __construct(private readonly ?\Exception $exception = null) {}

    /** @psalm-assert-if-true \Exception $this->getException() */
    public function hasException(): bool {
        return $this->exception !== null;
    }

    public function getException(): ?\Exception {
        return $this->exception;
    }
}

function example(Result $result): void {
    if ($result->hasException()) {
        echo $result->getException()->getMessage();
        echo $result->getException() === null ? 'null' : 'ex';
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'Exception' and 'null' is always false — these types can never be identical
    }
    echo $result->getException()->getMessage();
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method getMessage() on possibly null value
}

final class Box {
    private ?\Exception $exception = null;

    /** @psalm-assert-if-true \Exception $this->getException() */
    public function hasException(): bool {
        return $this->exception !== null;
    }

    public function getException(): ?\Exception {
        return $this->exception;
    }

    public function clear(): void {
        $this->exception = null;
    }
}

function boxed(Box $box): void {
    if ($box->hasException()) {
        echo $box->getException()->getMessage();
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method getMessage() on possibly null value
    }
}

final class Queue {
    /** @var list<\Exception> */
    private array $items = [];

    /** @psalm-assert-if-true \Exception $this->next() */
    public function hasNext(): bool {
        return $this->items !== [];
    }

    public function next(): ?\Exception {
        return array_shift($this->items);
    }
}

function draining(Queue $queue): void {
    if ($queue->hasNext()) {
        echo $queue->next()->getMessage();
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method getMessage() on possibly null value
    }
}

function mutating(Box $box): void {
    if ($box->hasException()) {
        $box->clear();
        echo $box->getException()->getMessage();
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method getMessage() on possibly null value
    }
}
