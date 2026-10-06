===description===
Docblocks from generic ancestors are not inherited, since their templates are unbound in the override.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
interface Box {
    /** @param T $v */
    public function put(mixed $v): void;
}
/** @implements Box<int> */
final class IntBox implements Box {
    public function put(mixed $v): void {
        /** @mir-check $v is mixed */
        $_x = $v;
    }
}
interface Tagged {
    /**
     * @template U
     * @param U $v
     */
    public function tag(mixed $v): void;
}
final class Tag implements Tagged {
    public function tag(mixed $v): void {
        /** @mir-check $v is mixed */
        $_x = $v;
    }
}
