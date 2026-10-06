===description===
A by-reference parameter without `@param-out` widens the caller's variable to the declared type after a method or static call, so keys the callee adds are readable. `@param-out` still wins, and without a call the empty array stays proven empty.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Collector {
    private function push(int $id, array &$out): void { $out[] = ['id' => $id]; }
    private static function pushStatic(int $id, array &$out): void { $out[] = ['id' => $id]; }
    /** @param list<int> $out */
    private function pushTyped(int $id, array &$out): void { $out[] = $id; }
    /** @param-out list<string> $out */
    private function pushOut(array &$out): void { $out = ['a']; }
    private function untyped(&$out): void { $out[] = 1; }

    public function viaInstance(): int {
        $path = [];
        $this->push(1, $path);
        /** @mir-check $path is array<array-key, mixed> */
        return $path[0]['id'];
    }

    public function viaStatic(): int {
        $path = [];
        self::pushStatic(1, $path);
        return $path[0]['id'];
    }

    public function viaDocblockType(): int {
        $path = [];
        $this->pushTyped(1, $path);
        /** @mir-check $path is list<int> */
        return $path[0];
    }

    public function paramOutWins(): string {
        $path = [];
        $this->pushOut($path);
        /** @mir-check $path is list<string> */
        return $path[0];
    }

    public function namedArgument(): int {
        $path = [];
        $this->push(out: $path, id: 1);
        return $path[0]['id'];
    }

    public function untypedKeepsCallerType(): void {
        $path = [];
        $this->untyped($path);
        /** @mir-check $path is array{} */
        $_ = $path;
    }

    public function noCallStaysEmpty(): int {
        $path = [];
        return $path[0];
//                   ^ NonExistentArrayOffset: Array offset '0' does not exist
    }
}

class Registry {
    /**
     * @template T
     * @param T $seed
     * @param array<int, T> $bucket
     */
    public function fill(mixed $seed, array &$bucket): void { $bucket[] = $seed; }

    public function templated(): void {
        $bucket = [];
        $this->fill('x', $bucket);
        /** @mir-check $bucket is array<int, "x"> */
        $_ = $bucket;
    }
}
