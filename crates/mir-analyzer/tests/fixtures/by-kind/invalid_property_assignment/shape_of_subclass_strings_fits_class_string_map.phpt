===description===
An array shape of subclass class-strings fits an `array<string, class-string<Parent>>` property
===file===
<?php
class Node {}
class ArrayNode extends Node {}
class Other {}

final class Builder {
    /** @var array<string, class-string<Node>> */
    private array $mapping = [];
    /** @var non-empty-array<string, class-string<Node>> */
    private array $nonEmpty = ['array' => ArrayNode::class];

    public function __construct() {
        $this->mapping = ['array' => ArrayNode::class];
        $this->nonEmpty = ['array' => ArrayNode::class];
        $this->mapping = ['other' => Other::class];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $mapping expects 'array<string, class-string<Node>>', cannot assign 'array{'other': class-string<Other>}'
    }

    public function get(): array {
        return [$this->mapping, $this->nonEmpty];
    }
}
