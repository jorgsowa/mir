===description===
A variable assigned under a condition on a property (`$this->lazy`, `$p->root !== ''`) is defined wherever the same condition holds again, even when the defining branch writes the property. A write to the property or an `unset()` of one of its elements between the two conditions, or a call that may write it, keeps it possibly undefined.
===file===
<?php
final class Path {
    public string $root = '';
}

function rootDepth(Path $p): void {
    if ($p->root !== '') { $depth = strlen($p->root); }
    if ($p->root !== '' && $depth > 1) { echo $depth; }
}

final class Form {
    /** @var array<string, string>|null */
    public ?array $errors = null;

    public function validate(): void {
        if (!$this->errors) {
            $version = '7';
            if (rand(0, 1)) {
                $this->errors['version'] = 'unknown';
            }
        }
        if (!$this->errors) { echo $version; }
    }

    public function elementUnsetBetween(): void {
        if ($this->errors) { $count = count($this->errors); }
        unset($this->errors['form']);
        if ($this->errors) { echo $count; }
//                                ^^^^^^ PossiblyUndefinedVariable: Variable $count might not be defined
    }
}

final class Session {
    public bool $lazy = false;

    public function start(): void {
        $this->lazy = false;
    }

    public function write(): void {
        if ($this->lazy) { $data = 'x'; }
        if ($this->lazy) { echo $data; }
    }

    public function writeAfterAssignment(): void {
        if ($this->lazy) { $data = 'x'; }
        $this->lazy = true;
        if ($this->lazy) { echo $data; }
//                              ^^^^^ PossiblyUndefinedVariable: Variable $data might not be defined
    }

    public function writeAfterCall(): void {
        if ($this->lazy) { $data = 'x'; }
        $this->start();
        if ($this->lazy) { echo $data; }
//                              ^^^^^ PossiblyUndefinedVariable: Variable $data might not be defined
    }
}
