===description===
An unannotated method is not treated as mutation-free when its body can write
a property: a direct write, an increment, a by-ref parameter, an unknown call,
or a call to another method that does.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Session {
    private ?string $user = null;
    private int $hits = 0;

    public function directWrite(string $u): void {
        $this->user = $u;
        $this->clear();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function viaCaller(string $u): void {
        $this->user = $u;
        $this->clearVia();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function viaUnknownCall(string $u): void {
        $this->user = $u;
        $this->callsUnknown();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function viaByRefParam(string $u): void {
        $this->user = $u;
        $this->fill($this->user);
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    public function viaIncrement(string $u): void {
        $this->user = $u;
        $this->bump();
        /** @mir-check $this->user is string|null */
        $_ = 1;
    }

    private function clear(): void {
        $this->user = null;
    }

    private function clearVia(): void {
        $this->clear();
    }

    private function callsUnknown(): void {
        helper($this);
    }

    private function fill(?string &$out): void {
        $out = null;
    }

    private function bump(): void {
        $this->hits++;
    }
}

function helper(Session $s): void {}
