===description===
`self` nested in list/array/shape/generic arguments resolves to the declaring class
===file===
<?php
class Node {
    /** @return list<self> */
    public static function pair(): array {
        return [new Node(), new Node()];
    }

    /** @return non-empty-list<self> */
    public static function nonEmpty(): array {
        return [new Node()];
    }

    /** @return array<string, self> */
    public static function byName(): array {
        return ['a' => new Node()];
    }

    /** @return array{first: self, rest: list<self>} */
    public static function shape(): array {
        return ['first' => new Node(), 'rest' => []];
    }

    /** @return \ArrayObject<int, self> */
    public function wrap(): \ArrayObject {
        return new \ArrayObject([$this]);
    }
}

/** @mir-check $pair is list<Node> */
$pair = Node::pair();
//<^^^^^^^^^^^^^^^^^^^^^ TypeCheckMismatch: Type of $pair is expected to be list<Node>, got mixed
echo count($pair);
