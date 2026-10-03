===description===
The nearest ancestor with a @param docblock wins across a class chain, and parameters are matched by position.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Source {
    /** @param non-empty-string $name */
    public function read(string $name, int $flags): void;
}
abstract class Base implements Source {
    public function read(string $name, int $flags): void {}
}
final class Leaf extends Base {
    public function read(string $title, int $flags): void {
        /** @mir-check $title is non-empty-string */
        $_a = $title;
        /** @mir-check $flags is int */
        $_b = $flags;
    }
}
===expect===
