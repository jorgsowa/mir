===description===
A `: int` private/final method or function whose body returns only literals keeps that union at the call site.
===config===
suppress=UnusedVariable
===file===
<?php
enum Lvl {
    case A; case B; case C;

    private function level(): int {
        return match ($this) { self::A => 0, self::B => 1, self::C => 2 };
    }

    final public function finalLevel(): int {
        return match ($this) { self::A => 0, self::B => 1, self::C => 2 };
    }

    public function exhaustive(): string {
        $l = $this->level();
        /** @mir-check $l is 0|1|2 */
        return match ($l) { 0 => 'a', 1 => 'b', 2 => 'c' };
    }

    public function missingArm(): string {
        return match ($this->level()) { 0 => 'a', 1 => 'b' };
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnhandledMatchCondition: Unhandled match condition: 2
    }

    public function finalMethod(): string {
        return match ($this->finalLevel()) { 0 => 'a', 1 => 'b', 2 => 'c' };
    }

    public function overridable(): int {
        return match ($this) { self::A => 0, self::B => 1, self::C => 2 };
    }

    public function overridableStaysInt(): string {
        $l = $this->overridable();
        /** @mir-check $l is int */
        return 'x';
    }

    private function single(): int {
        return 1;
    }

    public function singleLiteralStaysInt(): void {
        $s = $this->single();
        /** @mir-check $s is int */
    }
}

function fnLevel(bool $b): int {
    return $b ? 0 : 1;
}

function viaFunction(): string {
    return match (fnLevel(true)) { 0 => 'a', 1 => 'b' };
}

function tag(bool $b): string {
    return $b ? 'x' : 'y';
}

class Str {
    private function kind(): string {
        return rand() ? 'x' : 'y';
    }

    public function f(): int {
        return match ($this->kind()) { 'x' => 1, 'y' => 2 };
    }
}
===expect===
