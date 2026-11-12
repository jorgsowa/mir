===description===
A comparison made impossible only by a docblock type (`@param`, `@var` property, `@return`) is a DocblockTypeContradiction; a native type keeps ImpossibleIdenticalComparison.
===file===
<?php
interface Constraint {}

/** @param string|Constraint $c */
function param($c): bool {
    return $c === null;
//         ^^^^^^^^^^^ DocblockTypeContradiction: Type 'string|Constraint' makes '$c === null' impossible — this can never hold
}

/** @return string */
function docReturn() { return 'x'; }

function returned(): bool {
    return docReturn() !== null;
//         ^^^^^^^^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'string' makes 'docReturn() !== null' impossible — this can never hold
}

final class Holder {
    /** @var string */
    private $name;
    /** @var Constraint */
    private static $shared;

    public function __construct() {
        $this->name = 'x';
    }

    public function property(): bool {
        return $this->name === null;
//             ^^^^^^^^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'string' makes '$this->name === null' impossible — this can never hold
    }

    public static function staticProperty(): bool {
        return self::$shared != null;
//             ^^^^^^^^^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'Constraint' makes 'self::$shared != null' impossible — this can never hold
    }
}

function native(string $s): bool {
    return $s === null;
//         ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}
