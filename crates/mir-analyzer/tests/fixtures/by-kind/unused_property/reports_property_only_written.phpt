===description===
A private property holding plain data that is only assigned with `=` is never read
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Counter {
    private int $writeOnly = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Counter::$writeOnly is never read
    private static string $staticWriteOnly = '';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Counter::$staticWriteOnly is never read
    /** @var list<string> */
    private array $listWriteOnly = [];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Counter::$listWriteOnly is never read
    private int $read = 0;
    private int $compound = 0;
    private int $incremented = 0;
    private ?\Closure $keepsObjectAlive = null;
    private int $destructured = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Counter::$destructured is never read
    private int $promotedWritten;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Counter::$promotedWritten is never read

    public function __construct(private int $promoted) {
        $this->promotedWritten = 1;
    }

    public function run(): int {
        $this->writeOnly = 1;
        self::$staticWriteOnly = 'x';
        $this->listWriteOnly = ['a'];
        $this->read = 2;
        $this->compound += 1;
        $this->incremented++;
        $this->keepsObjectAlive = static fn() => null;
        [$this->destructured] = [1];
        return $this->read + $this->promoted;
    }
}

(new Counter(1))->run();
