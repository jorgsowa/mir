===description===
@inheritDoc overrides still inherit refined param types.
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
final class MoverImpl implements Mover {
    /** {@inheritDoc} */
    public function move(int $id): void {
        /** @mir-check $id is positive-int */
        $_x = $id;
    }
}
