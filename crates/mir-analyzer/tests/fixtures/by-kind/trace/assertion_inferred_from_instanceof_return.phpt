===description===
A function or method whose whole body is `return $p instanceof A || $p instanceof B;` narrows its argument at call sites as `instanceof A || instanceof B` would; other shapes infer nothing.
===file===
<?php
namespace App;

use Closure as Callback;

interface Node {}
final class Leaf implements Node {}
class Branch {}

function isLeaf(mixed $v): bool { return $v instanceof Leaf; }
function isCallbackOrLeaf(mixed $v): bool { return ($v instanceof Callback) || $v instanceof \App\Leaf; }
function isNode(mixed $v): bool { return $v instanceof Node; }
function isEither(mixed $a, mixed $b): bool { return $a instanceof Leaf || $b instanceof Leaf; }
function isBoth(mixed $v): bool { return $v instanceof Node && $v instanceof Leaf; }
function isDynamic(mixed $v, string $class): bool { return $v instanceof $class; }
function isLeafVariadic(mixed ...$v): bool { return $v instanceof Leaf; }
function isLeafLogged(mixed $v): bool { echo 1; return $v instanceof Leaf; }
function isLeafOrMissing(mixed $v): bool { return $v instanceof Leaf || $v instanceof Missing; }
//                                                                                    ^^^^^^^ UndefinedClass: Class App\Missing does not exist
function isMissing(mixed $v): bool { return $v instanceof Missing; }
//                                                        ^^^^^^^ UndefinedClass: Class App\Missing does not exist
/** @psalm-assert-if-true Node $v */
function isDeclared(mixed $v): bool { return $v instanceof Leaf; }

class Tree {
    protected function isSelf(mixed $v): bool { return $v instanceof self; }
    protected function isStatic(mixed $v): bool { return $v instanceof static; }
    protected function isThis(): bool { return $this instanceof Leaf; }

    public function self(int|Leaf|Tree $v): void {
        if ($this->isSelf($v)) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is App\Tree
        } else {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is int|App\Leaf
        }
    }

    public function static(int|Leaf|Tree $v): void {
        if ($this->isStatic($v)) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is int|App\Leaf|App\Tree
        }
    }

    public function this(int|Leaf|Tree $v): void {
//                       ^^^^^^^^^^^^^^^^ UnusedParam: Parameter $v is never used
        if ($this->isThis()) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is int|App\Leaf|App\Tree
        }
    }
}

final class Forest extends Tree {
    protected function isParent(mixed $v): bool { return $v instanceof parent; }

    public function parent(int|Tree $v): void {
        if ($this->isParent($v)) {
            /** @trace $v */
            echo 1;
//          ^^^^^^^ Trace: Type of $v is int|App\Tree
        }
    }
}

function one(int|Leaf|\Closure $v): void {
    if (isLeaf($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is App\Leaf
    } else {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|Closure
    }
}

function chain(int|Leaf|\Closure $v): void {
    if (isCallbackOrLeaf($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is Closure|App\Leaf
    } else {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int
    }
}

function keepsWhatIsKnown(int|Leaf|Branch|\Closure $v): void {
    if (isNode($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is App\Leaf|App\Branch&App\Node
    }
}

function twoParams(int|Leaf $v, int|Leaf $w): void {
    if (isEither($v, $w)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|App\Leaf
    }
}

function conjunction(int|Leaf $v): void {
    if (isBoth($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|App\Leaf
    }
}

function dynamic(int|Leaf $v): void {
    if (isDynamic($v, Leaf::class)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|App\Leaf
    }
}

function variadic(int|Leaf $v): void {
    if (isLeafVariadic($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|App\Leaf
    }
}

function moreThanReturn(int|Leaf $v): void {
    if (isLeafLogged($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is int|App\Leaf
    }
}

function someUnknown(int|Leaf $v): void {
    if (isLeafOrMissing($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is App\Leaf
    }
}

function allUnknown(int|Leaf $v): void {
    if (isMissing($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is App\Missing
    }
}

function declared(int|Leaf|Branch $v): void {
    if (isDeclared($v)) {
        /** @trace $v */
        echo 1;
//      ^^^^^^^ Trace: Type of $v is App\Leaf|App\Branch&App\Node
    }
}
