===description===
Interface and enum class constants must resolve to their declared/inferred
type (native type hint, @var docblock, or the literal value) instead of
always being `mixed` — collector/interface.rs and collector/enum.rs
hardcoded `Type::mixed()` for every ClassConst, never running the same
docblock/hint/literal resolution chain collector/class.rs already uses for
plain classes.
===file===
<?php

interface IBase {
    const LIMIT = 10;
}
class Impl implements IBase {}
function fromInterfaceConst(): int {
    $x = Impl::LIMIT;
    /** @mir-check $x is int */
    return $x;
}
function fromSelfInterfaceConst(): int {
    $x = IBase::LIMIT;
    /** @mir-check $x is int */
    return $x;
}

enum Status {
    case Active;
    case Inactive;
    const DEFAULT_LIMIT = 20;
}
function fromEnumConst(): int {
    $y = Status::DEFAULT_LIMIT;
    /** @mir-check $y is int */
    return $y;
}
===expect===
