===description===
An unannotated method is not treated as mutation-free when its body contains any
construct that may write a property or run unknown code, so the caller's
`$this->prop` narrowing is dropped after the call.
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
class Other { public function touch(): void {} public static function make(): void {} }

class Session {
    private ?string $user = null;
    private int $hits = 0;
    /** @var list<int> */
    private array $items = [];

    public function unsetsPropertyCaller(string $u): void {
        $this->user = $u;
        $this->unsetsProperty();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function foreachIntoPropertyCaller(string $u): void {
        $this->user = $u;
        $this->foreachIntoProperty();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function usesGlobalCaller(string $u): void {
        $this->user = $u;
        $this->usesGlobal();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function usesStaticVarCaller(string $u): void {
        $this->user = $u;
        $this->usesStaticVar();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function declaresFunctionCaller(string $u): void {
        $this->user = $u;
        $this->declaresFunction();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function assignsByRefCaller(string $u): void {
        $this->user = $u;
        $this->assignsByRef();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function preIncrementsPropertyCaller(string $u): void {
        $this->user = $u;
        $this->preIncrementsProperty();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function postIncrementsPropertyCaller(string $u): void {
        $this->user = $u;
        $this->postIncrementsProperty();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function passesPropertyToBuiltinCaller(string $u): void {
        $this->user = $u;
        $this->passesPropertyToBuiltin();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function callsImpureBuiltinCaller(string $u): void {
        $this->user = $u;
        $this->callsImpureBuiltin();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function callsOtherReceiverCaller(string $u): void {
        $this->user = $u;
        $this->callsOtherReceiver();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function callsNullsafeCaller(string $u): void {
        $this->user = $u;
        $this->callsNullsafe();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function callsDynamicStaticCaller(string $u): void {
        $this->user = $u;
        $this->callsDynamicStatic();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function createsObjectCaller(string $u): void {
        $this->user = $u;
        $this->createsObject();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function clonesObjectCaller(string $u): void {
        $this->user = $u;
        $this->clonesObject();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function includesFileCaller(string $u): void {
        $this->user = $u;
        $this->includesFile();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function evalsCaller(string $u): void {
        $this->user = $u;
        $this->evals();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function runsShellCaller(string $u): void {
        $this->user = $u;
        $this->runsShell();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function usesVariableVariableCaller(string $u): void {
        $this->user = $u;
        $this->usesVariableVariable();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function buildsClosureCaller(string $u): void {
        $this->user = $u;
        $this->buildsClosure();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function buildsArrowFunctionCaller(string $u): void {
        $this->user = $u;
        $this->buildsArrowFunction();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function buildsAnonymousClassCaller(string $u): void {
        $this->user = $u;
        $this->buildsAnonymousClass();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function buildsCallableCaller(string $u): void {
        $this->user = $u;
        $this->buildsCallable();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function callsStaticOnOtherCaller(string $u): void {
        $this->user = $u;
        $this->callsStaticOnOther();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    private function unsetsProperty(): void {
        unset($this->items[0]);
    }

    private function foreachIntoProperty(): void {
        foreach ([1] as $this->hits) {}
    }

    private function usesGlobal(): void {
        global $g;
    }

    private function usesStaticVar(): void {
        static $n = 0;
    }

    private function declaresFunction(): void {
        function inner(): void {}
    }

    private function assignsByRef(): void {
        $a = 1; $b = &$a;
    }

    private function preIncrementsProperty(): void {
        ++$this->hits;
    }

    private function postIncrementsProperty(): void {
        $this->hits++;
    }

    private function passesPropertyToBuiltin(): void {
        count($this->items);
    }

    private function callsImpureBuiltin(): void {
        file_put_contents('x', 'y');
    }

    private function callsOtherReceiver(): void {
        $o = new Other(); $o->touch();
    }

    private function callsNullsafe(): void {
        $o = null; $o?->touch();
    }

    private function callsDynamicStatic(): void {
        $m = 'x'; self::$m();
    }

    private function createsObject(): void {
        new Other();
    }

    private function clonesObject(): void {
        clone $this;
    }

    private function includesFile(): void {
        include 'x.php';
    }

    private function evals(): void {
        eval('1;');
    }

    private function runsShell(): void {
        `ls`;
    }

    private function usesVariableVariable(): void {
        $n = 'a'; $$n = 1;
    }

    private function buildsClosure(): void {
        $f = function (): void {};
    }

    private function buildsArrowFunction(): void {
        $f = fn() => 1;
    }

    private function buildsAnonymousClass(): void {
        new class {};
    }

    private function buildsCallable(): void {
        $f = strlen(...);
    }

    private function callsStaticOnOther(): void {
        Other::make();
    }
}
