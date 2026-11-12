===description===
A call to a method whose body only writes `$this` properties invalidates just those
properties, also through the methods it calls: one only appended to with `.=` widens to
`string`, one assigned otherwise falls back to its declared type, and the rest keep their
narrowing. Calls on other objects can't reach `$this` unless it is passed along. A memoized
call result is dropped, since it may read any property.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Logger {
    public function log(string $message): void {}
    public function attach(object $owner): void {}
}

final class Dumper {
    private Logger $logger;
    private ?string $reference = null;
    private ?int $depth = null;
    private ?string $name = null;

    public function dump(): string {
        $this->reference = '';
        $this->depth = 0;
        $this->writeNode();
        /** @mir-check $this->reference is string */
        $ref = $this->reference;
        /** @mir-check $this->depth is 0 */
        $this->reference = null;
        return $ref;
    }

    public function resetDepth(): int {
        $this->depth = 1;
        $this->name = 'x';
        $this->reset();
        /** @mir-check $this->name is "x" */
        return $this->depth;
//      ^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
    }

    public function memoized(): string {
        if ($this->label() === null) {
            return '';
        }
        $this->reset();
        return $this->label();
//      ^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    public function __construct() {
        $this->logger = new Logger();
    }

    public function logs(): string {
        $this->name = 'x';
        $this->logOnly();
        return $this->name;
    }

    public function escapes(): string {
        $this->name = 'x';
        $this->handOver();
        return $this->name;
//      ^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    private function logOnly(): void {
        $this->logger->log('dump');
    }

    private function handOver(): void {
        $this->logger->attach($this);
    }

    private function writeNode(): void {
        $this->writeLine('node');
    }

    private function writeLine(string $text): void {
        $this->reference .= $text;
    }

    private function reset(): void {
        $this->depth = null;
    }

    private function label(): ?string {
        return $this->depth === null ? null : $this->name;
    }
}
