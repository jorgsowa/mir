===description===
An offset write to a nullable array property creates the array, also when the narrowing was dropped by a call.
===file===
<?php
final class T {
    private ?array $p = null;
    private ?array $q = null;
    public function f(): array {
        $this->p['k'] = 'x';
        return $this->p;
    }
    public function g(): array {
        if ($this->q === null) { $this->q = []; }
        $this->q['k'] = $this->h();
        return $this->q;
    }
    private function h(): string { return 'x'; }
}
