===description===
`self` in a declared property type names the declaring class, so a subclass
instance is assignable (and narrows the property); an unrelated class is not.
===file===
<?php
abstract class Node {
    private static ?self $shared = null;
    private ?self $next = null;

    public static function shared(): Node {
        if (null === self::$shared) {
            self::$shared = new Leaf();
        }
        /** @mir-check self::$shared is Leaf|self(Node) */
        return self::$shared;
    }

    public function next(): Node {
        if ($this->next === null) {
            $this->next = new Leaf();
        }
        /** @mir-check $this->next is Leaf|self(Node) */
        return $this->next;
    }

    public function link(self $other): self { return $other; }

    public function linkLeaf(Leaf $leaf): Node {
        return $this->link($leaf);
    }

    public function unrelated(Other $o): void {
        $this->next = $o;
//      ^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $next expects 'self(Node)|null', cannot assign 'Other'
    }
}
final class Leaf extends Node {}
final class Other {}