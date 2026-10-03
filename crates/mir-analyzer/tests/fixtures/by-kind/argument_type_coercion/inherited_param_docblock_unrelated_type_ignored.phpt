===description===
An ancestor docblock that does not refine the native hint is ignored.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Reader {
    /** @param positive-int $id */
    public function read(string $id): void;
}
final class FileReader implements Reader {
    public function read(string $id): void {
        /** @mir-check $id is string */
        $_x = $id;
    }
}
===expect===
