<?php
namespace PhpParser;

// Recognizes `namespace X; ... class Y` well enough for the fixture files.
class ParserFactory
{
    public function createForHostVersion(): object
    {
        return new class {
            public function parse(string $code): array
            {
                $namespace = preg_match('/^namespace\s+([\w\\\\]+);/m', $code, $m) === 1 ? $m[1] . '\\' : '';
                $nodes = [];
                if (preg_match_all('/^(?:final\s+)?class\s+(\w+)/m', $code, $classes) > 0) {
                    foreach ($classes[1] as $name) {
                        $node = new \PhpParser\Node\Stmt\ClassLike();
                        $node->namespacedName = new class($namespace . $name) {
                            public function __construct(private string $name)
                            {
                            }

                            public function toString(): string
                            {
                                return $this->name;
                            }
                        };
                        $nodes[] = $node;
                    }
                }
                return $nodes;
            }
        };
    }
}
