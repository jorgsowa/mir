===description===
`self` nested in a property docblock type resolves to the declaring class when the property is read, assigned or inherited.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Node {
    /** @var list<self> */
    public array $children = [];
    /** @var array<string, self>|null */
    public ?array $byName = null;

    /** @param list<self> $promoted */
    public function __construct(
        public array $promoted = [],
    ) {}

    /** @return list<self> */
    public function kids(): array {
        return $this->children;
    }

    public function adopt(Node $child): void {
        $this->children[] = $child;
        $this->children = [$child];
        $this->byName = ['a' => $child];
    }
}

class SubNode extends Node {}
class Other {}

function takesNode(Node $n): void {}
function takesOther(Other $o): void {}

class Reader {
    public function read(Node $n, SubNode $s): void {
        $direct = $n->children;
        /** @mir-check $direct is list<self(Node)> */
        $_ = $direct;
        $inherited = $s->children;
        /** @mir-check $inherited is list<self(Node)> */
        $_ = $inherited;
        $promoted = $n->promoted;
        /** @mir-check $promoted is list<self(Node)> */
        $_ = $promoted;
        $named = $n->byName;
        /** @mir-check $named is array<string, self(Node)>|null */
        $_ = $named;
        foreach ($n->children as $child) {
            takesNode($child);
        }
        foreach ($s->children as $child) {
            takesNode($child);
        }
        foreach ($n->promoted as $child) {
            takesNode($child);
        }
        foreach ($n->byName ?? [] as $child) {
            takesNode($child);
        }
        foreach ($n->children as $child) {
            takesOther($child);
//                     ^^^^^^ InvalidArgument: Argument $o of takesOther() expects 'Other', got 'self(Node)'
        }
    }
}
===expect===
