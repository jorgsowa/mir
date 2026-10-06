===description===
An implementation without its own docblock inherits the interface's @param type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Mover {
    /** @param positive-int $id */
    public function move(int $id): void;
}
final class Event {
    /** @param list<positive-int> $ids */
    public function __construct(public array $ids) {}
}
final class MoverImpl implements Mover {
    public function move(int $id): void {
        /** @mir-check $id is positive-int */
        new Event([$id]);
    }
}
