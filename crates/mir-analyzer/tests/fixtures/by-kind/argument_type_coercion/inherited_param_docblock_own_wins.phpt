===description===
An override's own @param docblock and non-refining native hints are never replaced by the ancestor's.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {
    /** @param positive-int $a */
    public function f(int $a, string $b): void;
    /** @param positive-int $a */
    public function g(int|string $a): void;
}
final class Impl implements Shape {
    /** @param int<0, max> $a */
    public function f(int $a, string $b): void {
        /** @mir-check $a is int<0, max> */
        $_x = $a;
    }
    public function g(int|string $a): void {
        /** @mir-check $a is positive-int */
        $_x = $a;
    }
}
