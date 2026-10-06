===description===
An ancestor docblock naming an unknown class is not inherited.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Sink {
    /** @param list<Missing> $items */
    public function put(array $items): void;
}
final class ArraySink implements Sink {
    public function put(array $items): void {
        /** @mir-check $items is array<array-key, mixed> */
        $_x = $items;
    }
}
