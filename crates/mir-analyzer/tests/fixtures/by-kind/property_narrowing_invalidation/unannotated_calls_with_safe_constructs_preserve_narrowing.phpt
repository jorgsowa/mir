===description===
An unannotated method whose body only reads, writes locals, or calls pure builtins
and same-class static methods keeps the caller's `$this->prop` narrowing.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedMethod errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
    <UnusedFunctionCall errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Reader {
    private ?string $user = null;

    public function readsOnlyCaller(string $u): void {
        $this->user = $u;
        $this->readsOnly();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function unsetsLocalCaller(string $u): void {
        $this->user = $u;
        $this->unsetsLocal();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function foreachesLocalCaller(string $u): void {
        $this->user = $u;
        $this->foreachesLocal();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function incrementsLocalCaller(string $u): void {
        $this->user = $u;
        $this->incrementsLocal();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function assignsLocalArrayElementCaller(string $u): void {
        $this->user = $u;
        $this->assignsLocalArrayElement();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function callsPureBuiltinWithLocalsCaller(string $u): void {
        $this->user = $u;
        $this->callsPureBuiltinWithLocals();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    public function callsSelfStaticCaller(string $u): void {
        $this->user = $u;
        $this->callsSelfStatic();
        /** @mir-check $this->user is string */
        $_ = 1;
    }

    private function readsOnly(): int {
        return strlen((string) $this->user);
    }

    private function unsetsLocal(): int {
        $a = [1]; unset($a[0]); return 1;
    }

    private function foreachesLocal(): int {
        $t = 0; foreach ([1, 2] as $v) { $t += $v; } return $t;
    }

    private function incrementsLocal(): int {
        $i = 0; $i++; ++$i; return $i;
    }

    private function assignsLocalArrayElement(): int {
        $a = []; $a['k'] = 1; [$x, $y] = [1, 2]; return $x;
    }

    private function callsPureBuiltinWithLocals(): int {
        $s = 'abc'; return strlen(trim($s));
    }

    private function callsSelfStatic(): int {
        return self::answer() + static::answer();
    }

    private static function answer(): int {
        return 42;
    }
}
