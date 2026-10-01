<?php
namespace PhpParser;

class NodeFinder
{
    public function findFirst(array $nodes, callable $filter): ?object
    {
        foreach ($nodes as $node) {
            if ($filter($node)) {
                return $node;
            }
        }
        return null;
    }
}
