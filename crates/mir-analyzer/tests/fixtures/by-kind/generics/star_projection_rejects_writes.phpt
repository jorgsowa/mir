===description===
Through `Queue<*>` the element type is unknown, so a method taking the template cannot be called; methods that do not mention it can.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
final class Queue {
    /** @var list<T> */
    private array $items = [];

    /** @param T $item */
    public function push(mixed $item): void {
        $this->items[] = $item;
    }

    public function size(): int {
        return count($this->items);
    }
}

/** @param Queue<*> $queue */
function report(Queue $queue): int {
    $queue->push(1);
//               ^ InvalidArgument: Argument $item of push() expects 'never', got '1'
    $queue->push('job');
//               ^^^^^ InvalidArgument: Argument $item of push() expects 'never', got '"job"'
    return $queue->size();
}

/** @param Queue<int> $queue */
function fill(Queue $queue): void {
    $queue->push(1);
}
