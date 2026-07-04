===description===
int-mask-of<Other::FLAG_*> referencing a *different* class's constants is
not resolved (would need cross-file lookup unavailable during docblock
parsing) and falls back to plain `int` — no false positives.
===config===
suppress=UnusedParam
===file===
<?php
class Other {
    const FLAG_A = 1;
    const FLAG_B = 2;
}

class Flags {
    /**
     * @param int-mask-of<Other::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(999); // any int accepted — falls back to `int`
===expect===
