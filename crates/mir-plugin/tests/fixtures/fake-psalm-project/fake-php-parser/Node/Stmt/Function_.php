<?php
namespace PhpParser\Node\Stmt;

class Function_ implements \PhpParser\Node\FunctionLike
{
    use \PhpParser\Node\Attributes;

    public function __construct(public string $name)
    {
    }
}
