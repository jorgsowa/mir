<?php
namespace PhpParser\Node\Stmt;

class ClassMethod implements \PhpParser\Node\FunctionLike
{
    use \PhpParser\Node\Attributes;

    public function __construct(public string $name)
    {
    }
}
