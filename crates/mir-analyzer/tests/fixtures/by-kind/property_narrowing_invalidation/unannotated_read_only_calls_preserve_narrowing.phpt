===description===
An unannotated method whose body never writes a property (only reads, locals,
pure builtins and other such methods of the same class) keeps `$this->prop`
narrowing intact across the call, including through mutual recursion.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Session {
    private ?string $user = null;

    public function readOnlyCall(string $u): void {
        $this->user = $u;
        $this->label();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function transitiveCall(string $u): void {
        $this->user = $u;
        $this->wrapped();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function mutuallyRecursiveCall(string $u): void {
        $this->user = $u;
        $this->ping(2);
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    private function label(): string {
        $parts = [];
        $parts[] = strlen((string) $this->user);
        return implode(',', $parts);
    }

    private function wrapped(): string {
        return $this->label() . self::suffix();
    }

    private static function suffix(): string {
        return '!';
    }

    private function ping(int $n): int {
        return $n > 0 ? $this->pong($n - 1) : 0;
    }

    private function pong(int $n): int {
        return $this->ping($n);
    }
}
