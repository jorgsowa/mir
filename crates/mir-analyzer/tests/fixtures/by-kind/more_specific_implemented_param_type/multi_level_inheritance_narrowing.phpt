===description===
Each level of a three-level inheritance chain may narrow the @param docblock
further without emitting MethodSignatureMismatch. Only docblock types change;
native hints remain at the base type throughout.
===config===
suppress=UnusedParam
===file===
<?php
class Node {}
class TreeNode extends Node {}
class LeafNode extends TreeNode {}

class BaseVisitor {
    public function visit(Node $node): void {}
}

class TreeVisitor extends BaseVisitor {
    /** @param TreeNode $node */
    public function visit(Node $node): void {}
}

class LeafVisitor extends TreeVisitor {
    /** @param LeafNode $node */
    public function visit(Node $node): void {}
}
===expect===
