===description===
FALSE POSITIVE reproducer (J2, sibling manifestations). Narrowing a
variable TO one specific enum case via `===` must also satisfy a
bare-enum-typed property assignment and a bare-enum-typed array element,
via the same enum-case/bare-enum subtype arm exercised by the return-type
fixtures. Expected: no issue.
===config===
php_version=8.1
===file===
<?php
enum RoundingMode {
    case Unnecessary;
    case Up;
    case Down;
}

class Holder {
    public RoundingMode $mode;

    public function __construct(RoundingMode $mode) {
        $this->mode = $mode;
    }

    public function set(RoundingMode $mode): void {
        if ($mode === RoundingMode::Unnecessary) {
            $this->mode = $mode;
        }
    }
}

/**
 * @param list<RoundingMode> $modes
 * @return list<RoundingMode>
 */
function collect(array $modes, RoundingMode $mode): array {
    if ($mode === RoundingMode::Unnecessary) {
        $modes[] = $mode;
    }
    return $modes;
}
===expect===
