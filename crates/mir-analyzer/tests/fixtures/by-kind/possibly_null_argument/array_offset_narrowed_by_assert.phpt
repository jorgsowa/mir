===description===
M24: `assert($arr['key'] !== null)` / `if ($arr['key'] === null) { return; }`
now narrows a literal-keyed array-offset access's OWN value, same as an
`isset()` guard proves presence — reusing the existing shape-path
machinery (`narrow_shape_path`/`ShapeBase`) instead of a separate
refinement store, so the narrowed shape is picked up by a later
`$arr['key']` read for free. Without a guard, the same read is still
correctly flagged.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array{0: string, 1: ?string} $option */
function withAssert(array $option): array {
    assert($option[1] !== null);
    return explode('=', $option[1]);
}

/** @param array{0: string, 1: ?string} $option */
function withEarlyReturn(array $option): array {
    if ($option[1] === null) {
        return [];
    }
    return explode('=', $option[1]);
}

/** @param array{0: string, 1: ?string} $option */
function withoutGuard(array $option): array {
    return explode('=', $option[1]);
}
===expect===
PossiblyNullArgument@18:24-18:34: Argument $string of explode() might be null
